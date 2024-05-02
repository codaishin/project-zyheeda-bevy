use crate::{
	components::{SkillExecution, SlotVisibility},
	skill::SkillState,
	traits::{Execution, GetActiveSkill, GetAnimation, GetSlots},
};
use animations::traits::{InsertAnimation, MarkObsolete, Priority};
use behaviors::components::{Face, OverrideFace};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		system::{Commands, EntityCommands, Query, Res},
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

pub(crate) fn advance_active_skill<
	TGetSkill: GetActiveSkill<TAnimation, SkillState> + Component,
	TAnimation: Send + Sync + 'static,
	TAnimationDispatch: Component + InsertAnimation<TAnimation> + MarkObsolete<TAnimation>,
	TTime: Send + Sync + Default + 'static,
>(
	time: Res<Time<TTime>>,
	mut commands: Commands,
	mut agents: Query<(Entity, &mut TGetSkill, &mut TAnimationDispatch)>,
) {
	let delta = time.delta();

	for (entity, mut dequeue, mut animation_dispatch) in &mut agents {
		let Some(agent) = &mut commands.get_entity(entity) else {
			continue;
		};
		let dequeue = dequeue.as_mut();
		let dispatch = animation_dispatch.as_mut();

		if get_and_advance_skill(dequeue, agent, dispatch, delta) == Advancement::InProcess {
			continue;
		}

		dequeue.clear_active();
	}
}

fn get_and_advance_skill<
	TAnimation: Send + Sync + 'static,
	TGetSkill: GetActiveSkill<TAnimation, SkillState> + Sync + Send + 'static,
	TAnimationDispatch: InsertAnimation<TAnimation> + MarkObsolete<TAnimation>,
>(
	dequeue: &mut TGetSkill,
	agent: &mut EntityCommands,
	animation_dispatch: &mut TAnimationDispatch,
	delta: Duration,
) -> Advancement {
	match &mut dequeue.get_active() {
		None => remove_skill_side_effects(agent, animation_dispatch),
		Some(skill) => advance_skill(skill, agent, animation_dispatch, delta),
	}
}

fn remove_skill_side_effects<
	TAnimation: Send + Sync + 'static,
	TAnimationDispatch: MarkObsolete<TAnimation>,
>(
	agent: &mut EntityCommands,
	animation_dispatch: &mut TAnimationDispatch,
) -> Advancement {
	agent.remove::<OverrideFace>();
	animation_dispatch.mark_obsolete(Priority::High);

	Advancement::InProcess
}

fn advance_skill<
	TAnimation: Send + Sync + 'static,
	TAnimationDispatch: InsertAnimation<TAnimation> + MarkObsolete<TAnimation>,
>(
	skill: &mut (impl Execution + GetAnimation<TAnimation> + GetSlots + StateUpdate<SkillState>),
	agent: &mut EntityCommands,
	animation_dispatch: &mut TAnimationDispatch,
	delta: Duration,
) -> Advancement {
	let states = skill.update_state(delta);

	if states.contains(&StateMeta::First) {
		agent.try_insert(OverrideFace(Face::Cursor));
		agent.try_insert(SlotVisibility::Inherited(skill.slots()));
		begin_animation(skill, animation_dispatch);
	}

	if states.contains(&StateMeta::In(SkillState::Aim)) {
		agent.try_insert(OverrideFace(Face::Cursor));
	}

	if states.contains(&StateMeta::Leaving(SkillState::PreCast)) {
		insert_skill_execution_start(agent, skill);
	}

	if states.contains(&StateMeta::Leaving(SkillState::AfterCast)) {
		agent.try_insert(SlotVisibility::Hidden(skill.slots()));
		insert_skill_execution_stop(agent, skill);
		return Advancement::Finished;
	}

	Advancement::InProcess
}

fn begin_animation<TAnimation, TAnimationDispatch: InsertAnimation<TAnimation>>(
	skill: &mut (impl Execution + GetAnimation<TAnimation> + GetSlots + StateUpdate<SkillState>),
	dispatch: &mut TAnimationDispatch,
) {
	let Some(animation) = skill.animate() else {
		return;
	};
	dispatch.insert(animation, Priority::High);
}

fn insert_skill_execution_start<TSkill: Execution>(agent: &mut EntityCommands, skill: &mut TSkill) {
	let Some(start_fn) = skill.get_start() else {
		return;
	};
	agent.try_insert(SkillExecution::Start(start_fn));
}

fn insert_skill_execution_stop<TSkill: Execution>(agent: &mut EntityCommands, skill: &mut TSkill) {
	let Some(stop_fn) = skill.get_stop() else {
		return;
	};
	agent.try_insert(SkillExecution::Stop(stop_fn));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{SkillExecution, SlotKey, SlotVisibility},
		skill::{Spawner, StartBehaviorFn, StopBehaviorFn, Target},
		traits::{Execution, GetAnimation},
	};
	use animations::traits::Priority;
	use behaviors::components::{Face, OverrideFace};
	use bevy::{
		ecs::system::EntityCommands,
		prelude::{App, Transform, Update},
		time::{Real, Time},
	};
	use common::{
		components::Side,
		test_tools::utils::{SingleThreadedApp, TickTime},
	};
	use mockall::{mock, predicate::eq};
	use std::{collections::HashSet, time::Duration};

	#[derive(PartialEq)]
	enum BehaviorOption {
		Run,
		Stop,
	}

	#[derive(PartialEq)]
	enum MockOption {
		BehaviorExecution(BehaviorOption),
		Animate,
		Slot,
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

		if !no_setup.contains(&MockOption::BehaviorExecution(BehaviorOption::Run)) {
			mock.expect_get_start().return_const(None);
		}
		if !no_setup.contains(&MockOption::BehaviorExecution(BehaviorOption::Stop)) {
			mock.expect_get_stop().return_const(None);
		}
		if !no_setup.contains(&MockOption::Animate) {
			mock.expect_animate().return_const(None);
		}
		if !no_setup.contains(&MockOption::Slot) {
			mock.expect_slots().return_const(vec![]);
		}

		mock
	}

	impl GetActiveSkill<_Animation, SkillState> for _Dequeue {
		fn clear_active(&mut self) {
			self.active = None;
		}

		fn get_active(
			&mut self,
		) -> Option<impl Execution + GetAnimation<_Animation> + GetSlots + StateUpdate<SkillState>>
		{
			self.active.as_mut().map(|f| f())
		}
	}

	mock! {
		_Skill {}
		impl StateUpdate<SkillState> for _Skill {
			fn update_state(&mut self, delta: Duration) -> HashSet<StateMeta<SkillState>> {}
		}
		impl Execution for _Skill {
			fn get_start<'a>(&self) -> Option<StartBehaviorFn> {}
			fn get_stop<'a>(&self) -> Option<StopBehaviorFn> {}
		}
		impl GetAnimation<_Animation> for _Skill {
			fn animate(&self) -> Option<_Animation> {}
		}
		impl GetSlots for _Skill {
			fn slots(&self) -> Vec<SlotKey> {}
		}
	}

	#[derive(Component, Default)]
	struct _AnimationDispatch {
		mock: Mock_AnimationDispatch,
	}

	impl InsertAnimation<_Animation> for _AnimationDispatch {
		fn insert(&mut self, animation: _Animation, priority: Priority) {
			self.mock.insert(animation, priority)
		}
	}

	impl MarkObsolete<_Animation> for _AnimationDispatch {
		fn mark_obsolete(&mut self, priority: Priority) {
			self.mock.mark_obsolete(priority)
		}
	}

	mock! {
		_AnimationDispatch {}
		impl InsertAnimation<_Animation> for _AnimationDispatch {
			fn insert(&mut self, animation: _Animation, priority: Priority);
		}
		impl MarkObsolete<_Animation> for _AnimationDispatch {
			fn mark_obsolete(&mut self, priority: Priority);
		}
	}

	fn setup() -> (App, Entity) {
		let mut app = App::new().single_threaded(Update);
		let mut time = Time::<Real>::default();
		let mut dispatch = _AnimationDispatch::default();

		dispatch.mock.expect_insert().return_const(());
		dispatch.mock.expect_mark_obsolete().return_const(());
		let agent = app.world.spawn(dispatch).id();

		time.update();
		app.insert_resource(time);
		app.update();
		app.add_systems(
			Update,
			advance_active_skill::<_Dequeue, _Animation, _AnimationDispatch, Real>,
		);

		(app, agent)
	}

	#[test]
	fn call_update_with_delta() {
		let (mut app, agent) = setup();
		app.world.entity_mut(agent).insert((
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
	fn insert_animation_on_state_first() {
		let (mut app, agent) = setup();
		let mut dispatch = _AnimationDispatch::default();
		dispatch.mock.expect_mark_obsolete().return_const(());
		dispatch
			.mock
			.expect_insert()
			.times(1)
			.with(eq(_Animation(42)), eq(Priority::High))
			.return_const(());

		app.world.entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(move || {
					let mut skill = mock_skill_without_default_setup_for([MockOption::Animate]);
					skill
						.expect_update_state()
						.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::First]));
					skill.expect_animate().return_const(_Animation(42));
					skill
				})),
			},
			Transform::default(),
			dispatch,
		));

		app.update();
	}

	#[test]
	fn do_not_insert_animation_on_in_state_first() {
		let (mut app, agent) = setup();
		let mut dispatch = _AnimationDispatch::default();
		dispatch.mock.expect_mark_obsolete().return_const(());
		dispatch.mock.expect_insert().never().return_const(());

		app.world.entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(move || {
					let mut skill = mock_skill_without_default_setup_for([MockOption::Animate]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([
							StateMeta::In(SkillState::PreCast),
							StateMeta::Leaving(SkillState::PreCast),
							StateMeta::In(SkillState::Aim),
							StateMeta::Leaving(SkillState::Aim),
							StateMeta::In(SkillState::Active),
							StateMeta::Leaving(SkillState::Active),
							StateMeta::In(SkillState::AfterCast),
						]),
					);
					skill.expect_animate().return_const(_Animation(42));
					skill
				})),
			},
			Transform::default(),
			dispatch,
		));

		app.update();
	}

	#[test]
	fn set_slot_visible_on_first() {
		let (mut app, agent) = setup();
		app.world.entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([MockOption::Slot]);
					skill
						.expect_update_state()
						.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::First]));
					skill
						.expect_slots()
						.return_const(vec![SlotKey::Hand(Side::Main)]);
					skill
				})),
			},
			Transform::default(),
		));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&SlotVisibility::Inherited(vec![SlotKey::Hand(Side::Main)])),
			agent.get::<SlotVisibility>()
		);
	}

	#[test]
	fn set_multiple_slots_visible_on_first() {
		let (mut app, agent) = setup();
		app.world.entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([MockOption::Slot]);
					skill
						.expect_update_state()
						.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::First]));
					skill
						.expect_slots()
						.return_const(vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)]);
					skill
				})),
			},
			Transform::default(),
		));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&SlotVisibility::Inherited(vec![
				SlotKey::Hand(Side::Main),
				SlotKey::Hand(Side::Off)
			])),
			agent.get::<SlotVisibility>()
		);
	}

	#[test]
	fn hide_slot_when_done() {
		let (mut app, agent) = setup();
		app.world.entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([MockOption::Slot]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::Leaving(
							SkillState::AfterCast,
						)]),
					);
					skill
						.expect_slots()
						.return_const(vec![SlotKey::Hand(Side::Off)]);
					skill
				})),
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&SlotVisibility::Hidden(vec![SlotKey::Hand(Side::Off)])),
			agent.get::<SlotVisibility>()
		);
	}

	#[test]
	fn hide_multiple_slots_when_done() {
		let (mut app, agent) = setup();
		app.world.entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([MockOption::Slot]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::Leaving(
							SkillState::AfterCast,
						)]),
					);
					skill
						.expect_slots()
						.return_const(vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)]);
					skill
				})),
			},
			Transform::default(),
		));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&SlotVisibility::Hidden(vec![
				SlotKey::Hand(Side::Main),
				SlotKey::Hand(Side::Off)
			])),
			agent.get::<SlotVisibility>()
		);
	}

	#[test]
	fn remove_animation_when_no_active_skill() {
		let (mut app, agent) = setup();
		let mut dispatch = _AnimationDispatch::default();
		dispatch.mock.expect_insert().return_const(());
		dispatch
			.mock
			.expect_mark_obsolete()
			.times(1)
			.with(eq(Priority::High))
			.return_const(());

		app.world.entity_mut(agent).insert((
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
		dispatch.mock.expect_insert().return_const(());
		dispatch
			.mock
			.expect_mark_obsolete()
			.never()
			.return_const(());

		app.world.entity_mut(agent).insert((
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
	fn clear_queue_of_active() {
		let (mut app, agent) = setup();
		app.world.entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::Leaving(
							SkillState::AfterCast,
						)]),
					);
					skill
				})),
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.get::<_Dequeue>().unwrap().active.is_none());
	}

	#[test]
	fn do_not_remove_skill_when_not_done() {
		let (mut app, agent) = setup();
		app.world.entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
							SkillState::AfterCast,
						)]),
					);
					skill
				})),
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.get::<_Dequeue>().unwrap().active.is_some());
	}

	#[test]
	fn run() {
		fn start_behavior(_: &mut EntityCommands, _: &Transform, _: &Spawner, _: &Target) {}

		let (mut app, agent) = setup();
		app.world.entity_mut(agent).insert(_Dequeue {
			active: Some(Box::new(|| {
				let mut skill =
					mock_skill_without_default_setup_for([MockOption::BehaviorExecution(
						BehaviorOption::Run,
					)]);
				skill
					.expect_update_state()
					.return_const(HashSet::<StateMeta<SkillState>>::from([
						StateMeta::Leaving(SkillState::PreCast),
					]));
				skill.expect_get_start().returning(|| Some(start_behavior));
				skill
			})),
		});

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&SkillExecution::Start(start_behavior)),
			agent.get::<SkillExecution>()
		);
	}

	#[test]
	fn do_run_when_not_activating_this_frame() {
		let (mut app, agent) = setup();
		app.world.entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill =
						mock_skill_without_default_setup_for([MockOption::BehaviorExecution(
							BehaviorOption::Run,
						)]);

					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::In(SkillState::Active)]),
					);
					skill.expect_get_start().never().return_const(None);
					skill
				})),
			},
			Transform::default(),
		));

		app.update();
	}

	#[test]
	fn stop() {
		fn stop_fn(_: &mut EntityCommands) {}

		let (mut app, agent) = setup();
		app.world.entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill =
						mock_skill_without_default_setup_for([MockOption::BehaviorExecution(
							BehaviorOption::Stop,
						)]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::Leaving(
							SkillState::AfterCast,
						)]),
					);
					skill.expect_get_stop().returning(|| Some(stop_fn));
					skill
				})),
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&SkillExecution::Stop(stop_fn)),
			agent.get::<SkillExecution>()
		);
	}

	#[test]
	fn do_not_stop_when_not_done() {
		let (mut app, agent) = setup();
		app.world.entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill =
						mock_skill_without_default_setup_for([MockOption::BehaviorExecution(
							BehaviorOption::Stop,
						)]);

					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::In(SkillState::Active)]),
					);
					skill.expect_get_stop().never().return_const(None);
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
		app.world.entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([]);
					skill
						.expect_update_state()
						.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::First]));
					skill
				})),
			},
			Transform::default(),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&OverrideFace(Face::Cursor)),
			agent.get::<OverrideFace>()
		);
	}

	#[test]
	fn do_not_apply_facing_override_when_not_new() {
		let (mut app, agent) = setup();
		app.world.entity_mut(agent).insert((
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

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<OverrideFace>());
	}

	#[test]
	fn apply_apply_facing_override_when_aiming() {
		let (mut app, agent) = setup();
		app.world.entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::In(SkillState::Aim)]),
					);
					skill
				})),
			},
			Transform::from_xyz(-1., -2., -3.),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			Some(&OverrideFace(Face::Cursor)),
			agent.get::<OverrideFace>()
		);
	}

	#[test]
	fn no_facing_override_when_no_skill() {
		let (mut app, agent) = setup();
		app.world.entity_mut(agent).insert((
			_Dequeue { active: None },
			Transform::from_xyz(-1., -2., -3.),
			OverrideFace(Face::Cursor),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<OverrideFace>());
	}
}
