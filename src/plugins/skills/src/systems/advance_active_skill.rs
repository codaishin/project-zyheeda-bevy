use crate::{
	skills::{Animate, SkillBehavior, SkillState, StartBehaviorFn},
	traits::{Flush, GetActiveSkill, GetAnimation, GetSkillBehavior, Schedule},
};
use animations::traits::{SkillLayer, StartAnimation, StopAnimation};
use behaviors::components::{Face, OverrideFace};
use bevy::{
	ecs::{
		change_detection::DetectChanges,
		component::Component,
		entity::Entity,
		system::{Commands, EntityCommands, Query, Res},
		world::Mut,
	},
	time::Time,
};
use common::traits::state_duration::{StateMeta, StateUpdate};
use std::time::Duration;

#[derive(PartialEq)]
enum Advancement {
	Finished,
	InProcess,
}

type Components<'a, TGetSkill, TAnimationDispatch, TSkillExecutor> = (
	Entity,
	&'a mut TGetSkill,
	&'a mut TAnimationDispatch,
	&'a mut TSkillExecutor,
);

pub(crate) fn advance_active_skill<
	TGetSkill: GetActiveSkill<TAnimation, SkillState> + Component,
	TAnimation: Send + Sync + 'static,
	TAnimationDispatch: Component + StartAnimation<SkillLayer, TAnimation> + StopAnimation<SkillLayer>,
	TSkillExecutor: Component + Schedule + Flush,
	TTime: Send + Sync + Default + 'static,
>(
	time: Res<Time<TTime>>,
	mut commands: Commands,
	mut agents: Query<Components<TGetSkill, TAnimationDispatch, TSkillExecutor>>,
) {
	let delta = time.delta();

	for (entity, mut dequeue, animation_dispatch, skill_executer) in &mut agents {
		let Some(agent) = commands.get_entity(entity) else {
			continue;
		};
		let changed = dequeue.is_changed();
		let advancement = match dequeue.get_active() {
			Some(skill) => advance(skill, agent, animation_dispatch, skill_executer, delta),
			None if changed => remove_side_effects(agent, animation_dispatch),
			_ => Advancement::InProcess,
		};

		if advancement == Advancement::InProcess {
			continue;
		}

		dequeue.clear_active();
	}
}

fn remove_side_effects<TAnimationDispatch: StopAnimation<SkillLayer>>(
	mut agent: EntityCommands,
	mut animation_dispatch: Mut<TAnimationDispatch>,
) -> Advancement {
	agent.remove::<OverrideFace>();
	animation_dispatch.stop_animation();

	Advancement::InProcess
}

fn advance<
	TAnimation: Send + Sync + 'static,
	TAnimationDispatch: StartAnimation<SkillLayer, TAnimation> + StopAnimation<SkillLayer>,
	TSkillExecutor: Component + Schedule + Flush,
>(
	mut skill: (impl GetSkillBehavior + GetAnimation<TAnimation> + StateUpdate<SkillState>),
	mut agent: EntityCommands,
	mut animation_dispatch: Mut<TAnimationDispatch>,
	mut skill_executer: Mut<TSkillExecutor>,
	delta: Duration,
) -> Advancement {
	let skill = &mut skill;
	let agent = &mut agent;
	let animation_dispatch = animation_dispatch.as_mut();
	let skill_executer = skill_executer.as_mut();
	let states = skill.update_state(delta);

	if states.contains(&StateMeta::Entering(SkillState::Aim)) {
		agent.try_insert(OverrideFace(Face::Cursor));
		animate(skill, animation_dispatch);
		schedule_start(skill_executer, skill, start_on_aim);
	}

	if states.contains(&StateMeta::Entering(SkillState::Active)) {
		schedule_start(skill_executer, skill, start_on_active);
	}

	if states.contains(&StateMeta::Done) {
		skill_executer.flush();
		return Advancement::Finished;
	}

	Advancement::InProcess
}

fn animate<
	TAnimation,
	TAnimationDispatch: StartAnimation<SkillLayer, TAnimation> + StopAnimation<SkillLayer>,
>(
	skill: &mut (impl GetSkillBehavior + GetAnimation<TAnimation> + StateUpdate<SkillState>),
	dispatch: &mut TAnimationDispatch,
) {
	match skill.animate() {
		Animate::Some(animation) => dispatch.start_animation(animation),
		Animate::None => dispatch.stop_animation(),
		Animate::Ignore => {}
	}
}

fn start_on_aim<TSkill: GetSkillBehavior>(skill: &TSkill) -> Option<StartBehaviorFn> {
	match skill.behavior() {
		SkillBehavior::OnAim(run) => Some(run),
		_ => None,
	}
}

fn start_on_active<TSkill: GetSkillBehavior>(skill: &TSkill) -> Option<StartBehaviorFn> {
	match skill.behavior() {
		SkillBehavior::OnActive(run) => Some(run),
		_ => None,
	}
}

fn schedule_start<TSkillExecutor: Schedule, TSkill: GetSkillBehavior>(
	executer: &mut TSkillExecutor,
	skill: &TSkill,
	get_start_fn: fn(&TSkill) -> Option<StartBehaviorFn>,
) {
	let Some(start_fn) = get_start_fn(skill) else {
		return;
	};
	executer.schedule(start_fn);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		skills::OnSkillStop,
		traits::{GetAnimation, GetSkillBehavior},
	};
	use behaviors::components::{Face, OverrideFace};
	use bevy::{
		prelude::{App, Transform, Update},
		time::{Real, Time},
	};
	use common::test_tools::utils::{SingleThreadedApp, TickTime};
	use mockall::{mock, predicate::eq};
	use std::{collections::HashSet, time::Duration};

	#[derive(PartialEq)]
	enum MockOption {
		RunBehavior,
		Animate,
	}

	#[derive(Default, Debug, PartialEq, Clone, Copy)]
	struct _Animation(usize);

	#[derive(Component, Default)]
	struct _Dequeue {
		pub active: Option<Box<dyn FnMut() -> Mock_Skill + Sync + Send>>,
	}

	fn mock_skill_without_default_setup_for<const N: usize>(
		no_setup: [MockOption; N],
	) -> Mock_Skill {
		let mut mock = Mock_Skill::new();

		if !no_setup.contains(&MockOption::RunBehavior) {
			mock.expect_behavior().return_const(SkillBehavior::Never);
		}
		if !no_setup.contains(&MockOption::Animate) {
			mock.expect_animate().return_const(Animate::Ignore);
		}

		mock
	}

	impl GetActiveSkill<_Animation, SkillState> for _Dequeue {
		fn clear_active(&mut self) {
			self.active = None;
		}

		fn get_active(
			&mut self,
		) -> Option<impl GetSkillBehavior + GetAnimation<_Animation> + StateUpdate<SkillState>> {
			self.active.as_mut().map(|f| f())
		}
	}

	mock! {
		_Skill {}
		impl StateUpdate<SkillState> for _Skill {
			fn update_state(&mut self, delta: Duration) -> HashSet<StateMeta<SkillState>> {}
		}
		impl GetSkillBehavior for _Skill {
			fn behavior<'a>(&self) -> SkillBehavior {}
		}
		impl GetAnimation<_Animation> for _Skill {
			fn animate(&self) -> Animate<_Animation> {}
		}
	}

	#[derive(Component, Default)]
	struct _AnimationDispatch {
		mock: Mock_AnimationDispatch,
	}

	impl StartAnimation<SkillLayer, _Animation> for _AnimationDispatch {
		fn start_animation(&mut self, animation: _Animation) {
			self.mock.start_animation(animation)
		}
	}

	impl StopAnimation<SkillLayer> for _AnimationDispatch {
		fn stop_animation(&mut self) {
			self.mock.stop_animation()
		}
	}

	mock! {
		_AnimationDispatch {}
		impl StartAnimation<SkillLayer, _Animation> for _AnimationDispatch {
			fn start_animation(&mut self, animation: _Animation);
		}
		impl StopAnimation<SkillLayer> for _AnimationDispatch {
			fn stop_animation(&mut self);
		}
	}

	#[derive(Component, Default)]
	struct _Executor {
		mock: Mock_Executor,
	}

	impl Schedule for _Executor {
		fn schedule(&mut self, start: StartBehaviorFn) {
			self.mock.schedule(start)
		}
	}

	impl Flush for _Executor {
		fn flush(&mut self) {
			self.mock.flush()
		}
	}

	mock! {
		_Executor {}
		impl Schedule for _Executor {
			fn schedule(&mut self, start: StartBehaviorFn);
		}
		impl Flush for _Executor {
			fn flush(&mut self);
		}
	}

	const START_BEHAVIOR: StartBehaviorFn = |_, _, _, _| OnSkillStop::Ignore;

	fn setup() -> (App, Entity) {
		let mut app = App::new().single_threaded(Update);
		let mut time = Time::<Real>::default();
		let mut dispatch = _AnimationDispatch::default();
		let mut executer = _Executor::default();

		dispatch.mock.expect_start_animation().return_const(());
		dispatch.mock.expect_stop_animation().return_const(());
		executer.mock.expect_schedule().return_const(());
		executer.mock.expect_flush().return_const(());
		let agent = app.world_mut().spawn((dispatch, executer)).id();

		time.update();
		app.insert_resource(time);
		app.update();
		app.add_systems(
			Update,
			advance_active_skill::<_Dequeue, _Animation, _AnimationDispatch, _Executor, Real>,
		);

		(app, agent)
	}

	#[test]
	fn call_update_with_delta() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([]);
					skill
						.expect_update_state()
						.times(1)
						.with(eq(Duration::from_millis(100)))
						.return_const(HashSet::<StateMeta<SkillState>>::default());
					skill
				})),
			},
			Transform::default(),
		));

		app.tick_time(Duration::from_millis(100));
		app.update();
	}

	#[test]
	fn insert_animation_when_aim_begins() {
		let (mut app, agent) = setup();
		let mut dispatch = _AnimationDispatch::default();
		dispatch.mock.expect_stop_animation().return_const(());
		dispatch
			.mock
			.expect_start_animation()
			.times(1)
			.with(eq(_Animation(42)))
			.return_const(());

		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(move || {
					let mut skill = mock_skill_without_default_setup_for([MockOption::Animate]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
							SkillState::Aim,
						)]),
					);
					skill
						.expect_animate()
						.return_const(Animate::Some(_Animation(42)));
					skill
				})),
			},
			Transform::default(),
			dispatch,
		));

		app.update();
	}

	#[test]
	fn do_not_insert_animation_when_not_beginning_to_aim() {
		let (mut app, agent) = setup();
		let mut dispatch = _AnimationDispatch::default();
		dispatch.mock.expect_stop_animation().return_const(());
		dispatch
			.mock
			.expect_start_animation()
			.never()
			.return_const(());

		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(move || {
					let mut skill = mock_skill_without_default_setup_for([MockOption::Animate]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([
							StateMeta::In(SkillState::Aim),
							StateMeta::Entering(SkillState::Active),
							StateMeta::In(SkillState::Active),
							StateMeta::Done,
						]),
					);
					skill
						.expect_animate()
						.return_const(Animate::Some(_Animation(42)));
					skill
				})),
			},
			Transform::default(),
			dispatch,
		));

		app.update();
	}

	#[test]
	fn stop_animation_on_when_beginning_to_aim_and_animate_is_none() {
		let (mut app, agent) = setup();
		let mut dispatch = _AnimationDispatch::default();
		dispatch.mock.expect_start_animation().return_const(());
		dispatch
			.mock
			.expect_stop_animation()
			.times(1)
			.return_const(());

		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(move || {
					let mut skill = mock_skill_without_default_setup_for([MockOption::Animate]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
							SkillState::Aim,
						)]),
					);
					skill.expect_animate().return_const(Animate::None);
					skill
				})),
			},
			Transform::default(),
			dispatch,
		));

		app.update();
	}

	#[test]
	fn do_not_stop_animation_when_not_beginning_to_aim_and_animate_is_none() {
		let (mut app, agent) = setup();
		let mut dispatch = _AnimationDispatch::default();
		dispatch.mock.expect_start_animation().return_const(());
		dispatch
			.mock
			.expect_stop_animation()
			.never()
			.return_const(());

		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(move || {
					let mut skill = mock_skill_without_default_setup_for([MockOption::Animate]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([
							StateMeta::In(SkillState::Aim),
							StateMeta::Entering(SkillState::Active),
							StateMeta::In(SkillState::Active),
							StateMeta::Done,
						]),
					);
					skill.expect_animate().return_const(Animate::None);
					skill
				})),
			},
			Transform::default(),
			dispatch,
		));

		app.update();
	}

	#[test]
	fn do_not_start_or_stop_animation_when_beginning_to_aim_and_animate_is_ignore() {
		let (mut app, agent) = setup();
		let mut dispatch = _AnimationDispatch::default();
		dispatch
			.mock
			.expect_start_animation()
			.never()
			.return_const(());
		dispatch
			.mock
			.expect_stop_animation()
			.never()
			.return_const(());

		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(move || {
					let mut skill = mock_skill_without_default_setup_for([MockOption::Animate]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
							SkillState::Aim,
						)]),
					);
					skill.expect_animate().return_const(Animate::Ignore);
					skill
				})),
			},
			Transform::default(),
			dispatch,
		));

		app.update();
	}

	#[test]
	fn remove_animation_when_no_active_skill() {
		let (mut app, agent) = setup();
		let mut dispatch = _AnimationDispatch::default();
		dispatch.mock.expect_start_animation().return_const(());
		dispatch
			.mock
			.expect_stop_animation()
			.times(1)
			.return_const(());

		app.world_mut().entity_mut(agent).insert((
			_Dequeue { active: None },
			Transform::default(),
			dispatch,
		));

		app.update();
	}

	#[test]
	fn do_not_remove_animation_when_some_active_skill() {
		let (mut app, agent) = setup();
		let mut dispatch = _AnimationDispatch::default();
		dispatch.mock.expect_start_animation().return_const(());
		dispatch
			.mock
			.expect_stop_animation()
			.never()
			.return_const(());

		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([]);
					skill.expect_update_state().return_const(HashSet::default());
					skill
				})),
			},
			Transform::default(),
			dispatch,
		));

		app.update();
	}

	#[test]
	fn remove_animation_only_once_when_no_active_skill() {
		let (mut app, agent) = setup();
		let mut dispatch = _AnimationDispatch::default();
		dispatch.mock.expect_start_animation().return_const(());
		dispatch
			.mock
			.expect_stop_animation()
			.times(1)
			.return_const(());

		app.world_mut().entity_mut(agent).insert((
			_Dequeue { active: None },
			Transform::default(),
			dispatch,
		));

		app.update();
		app.update();
	}

	#[test]
	fn clear_queue_of_active() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([]);
					skill
						.expect_update_state()
						.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::Done]));
					skill
				})),
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world().entity(agent);

		assert!(agent.get::<_Dequeue>().unwrap().active.is_none());
	}

	#[test]
	fn do_not_remove_skill_when_not_done() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::In(SkillState::Active)]),
					);
					skill
				})),
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world().entity(agent);

		assert!(agent.get::<_Dequeue>().unwrap().active.is_some());
	}

	#[test]
	fn run_on_active() {
		let mut executor = _Executor::default();
		executor.mock.expect_flush().return_const(());
		executor
			.mock
			.expect_schedule()
			.times(1)
			.withf(|start| {
				assert_eq!(start, &START_BEHAVIOR);
				true
			})
			.return_const(());

		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			executor,
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([MockOption::RunBehavior]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
							SkillState::Active,
						)]),
					);
					skill
						.expect_behavior()
						.returning(|| SkillBehavior::OnActive(START_BEHAVIOR));
					skill
				})),
			},
		));

		app.update();
	}

	#[test]
	fn run_on_aim() {
		let mut executor = _Executor::default();
		executor.mock.expect_flush().return_const(());
		executor
			.mock
			.expect_schedule()
			.times(1)
			.withf(|start| {
				assert_eq!(start, &START_BEHAVIOR);
				true
			})
			.return_const(());

		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			executor,
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([MockOption::RunBehavior]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
							SkillState::Aim,
						)]),
					);
					skill
						.expect_behavior()
						.returning(|| SkillBehavior::OnAim(START_BEHAVIOR));
					skill
				})),
			},
		));

		app.update();
	}

	#[test]
	fn do_not_run_when_not_activating_this_frame() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([MockOption::RunBehavior]);

					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::In(SkillState::Active)]),
					);
					skill
						.expect_behavior()
						.never()
						.return_const(SkillBehavior::Never);
					skill
				})),
			},
			Transform::default(),
		));

		app.update();
	}

	#[test]
	fn flush() {
		let mut executor = _Executor::default();
		executor.mock.expect_schedule().return_const(());
		executor.mock.expect_flush().times(1).return_const(());

		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			executor,
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([]);
					skill
						.expect_update_state()
						.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::Done]));
					skill
				})),
			},
			Transform::default(),
		));

		app.update();
	}

	#[test]
	fn do_not_stop_when_not_done() {
		let mut executor = _Executor::default();
		executor.mock.expect_schedule().return_const(());
		executor.mock.expect_flush().never().return_const(());

		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			executor,
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([]);

					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::In(SkillState::Active)]),
					);
					skill
				})),
			},
			Transform::default(),
		));

		app.update();
	}

	#[test]
	fn apply_facing() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
							SkillState::Aim,
						)]),
					);
					skill
				})),
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&OverrideFace(Face::Cursor)),
			agent.get::<OverrideFace>()
		);
	}

	#[test]
	fn do_not_apply_facing_when_not_beginning_to_aim() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::In(SkillState::Aim)]),
					);
					skill
				})),
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<OverrideFace>());
	}

	#[test]
	fn apply_facing_override_when_beginning_to_aim() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::Entering(
							SkillState::Aim,
						)]),
					);
					skill
				})),
			},
			Transform::from_xyz(-1., -2., -3.),
		));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(
			Some(&OverrideFace(Face::Cursor)),
			agent.get::<OverrideFace>()
		);
	}

	#[test]
	fn no_facing_override_when_no_skill() {
		let (mut app, agent) = setup();
		app.world_mut().entity_mut(agent).insert((
			_Dequeue { active: None },
			Transform::from_xyz(-1., -2., -3.),
			OverrideFace(Face::Cursor),
		));

		app.update();

		let agent = app.world().entity(agent);

		assert_eq!(None, agent.get::<OverrideFace>());
	}
}
