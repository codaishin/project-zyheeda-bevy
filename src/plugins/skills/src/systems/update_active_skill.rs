use crate::{
	components::{SkillExecution, SlotVisibility},
	skill::SkillState,
	traits::{Execution, GetActiveSkill, GetAnimation, GetSlots},
};
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
enum State {
	Done,
	Busy,
}

pub(crate) fn update_active_skill<
	TAnimationKey: Copy + Clone + PartialEq + Send + Sync + 'static,
	TGetSkill: GetActiveSkill<TAnimationKey, SkillState> + Component,
	TTime: Send + Sync + Default + 'static,
>(
	time: Res<Time<TTime>>,
	mut commands: Commands,
	mut agents: Query<(Entity, &mut TGetSkill)>,
) {
	let delta = time.delta();

	for (entity, mut dequeue) in &mut agents {
		let Some(agent) = &mut commands.get_entity(entity) else {
			continue;
		};
		let dequeue = dequeue.as_mut();

		if advance_skill(agent, dequeue, delta) == State::Busy {
			continue;
		}

		dequeue.clear_active();
	}
}

fn advance_skill<
	TAnimationKey: Copy + Clone + PartialEq + Send + Sync + 'static,
	TGetSkill: GetActiveSkill<TAnimationKey, SkillState> + Sync + Send + 'static,
>(
	agent: &mut EntityCommands,
	dequeue: &mut TGetSkill,
	delta: Duration,
) -> State {
	let Some(skill) = &mut dequeue.get_active() else {
		return State::Busy;
	};

	let states = skill.update_state(delta);

	if states.contains(&StateMeta::First) {
		agent.try_insert(OverrideFace(Face::Cursor));
		agent.try_insert(SlotVisibility::Inherited(skill.slots()));
	}

	if states.contains(&StateMeta::In(SkillState::Aim)) {
		agent.try_insert(OverrideFace(Face::Cursor));
	}

	if states.contains(&StateMeta::Leaving(SkillState::PreCast)) {
		insert_skill_execution_start(agent, skill);
	}

	if states.contains(&StateMeta::Leaving(SkillState::AfterCast)) {
		agent.try_insert(SlotVisibility::Hidden(skill.slots()));
		agent.remove::<OverrideFace>();
		insert_skill_execution_stop(agent, skill);
		return State::Done;
	}

	agent.try_insert(skill.animate());

	State::Busy
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
		traits::Execution,
	};
	use behaviors::components::{Face, OverrideFace};
	use bevy::{
		ecs::system::EntityCommands,
		prelude::{App, Transform, Update},
		time::{Real, Time},
	};
	use common::{
		components::{Animate, Side},
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

	#[derive(Debug, PartialEq, Clone, Copy)]
	enum _AnimationKey {
		A,
	}

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
			mock.expect_animate().return_const(Animate::None);
		}
		if !no_setup.contains(&MockOption::Slot) {
			mock.expect_slots().return_const(vec![]);
		}

		mock
	}

	impl GetActiveSkill<_AnimationKey, SkillState> for _Dequeue {
		fn clear_active(&mut self) {
			self.active = None;
		}

		fn get_active(
			&mut self,
		) -> Option<impl Execution + GetAnimation<_AnimationKey> + GetSlots + StateUpdate<SkillState>>
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
		impl GetAnimation<_AnimationKey> for _Skill {
			fn animate(&self) -> Animate<_AnimationKey> {}
		}
		impl GetSlots for _Skill {
			fn slots(&self) -> Vec<SlotKey> {}
		}
	}

	fn setup() -> (App, Entity) {
		let mut app = App::new().single_threaded(Update);
		let mut time = Time::<Real>::default();
		let agent = app.world.spawn(()).id();

		time.update();
		app.insert_resource(time);
		app.update();
		app.add_systems(Update, update_active_skill::<_AnimationKey, _Dequeue, Real>);

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
	fn add_animation_on_each_state_except_when_done() {
		//FIXME: This needs to be some kind of fixture. Maybe try `rstest` crate?
		let states = [
			StateMeta::First,
			StateMeta::In(SkillState::PreCast),
			StateMeta::Leaving(SkillState::PreCast),
			StateMeta::In(SkillState::Aim),
			StateMeta::Leaving(SkillState::Aim),
			StateMeta::In(SkillState::Active),
			StateMeta::Leaving(SkillState::Active),
			StateMeta::In(SkillState::AfterCast),
		];

		for state in states {
			let (mut app, agent) = setup();

			app.world.entity_mut(agent).insert((
				_Dequeue {
					active: Some(Box::new(move || {
						let mut skill = mock_skill_without_default_setup_for([MockOption::Animate]);
						skill
							.expect_update_state()
							.return_const(HashSet::<StateMeta<SkillState>>::from([state.clone()]));
						skill
							.expect_animate()
							.return_const(Animate::Repeat(_AnimationKey::A));
						skill
					})),
				},
				Transform::default(),
			));
			app.update();

			let agent = app.world.entity(agent);

			assert_eq!(
				Some(&Animate::Repeat(_AnimationKey::A)),
				agent.get::<Animate<_AnimationKey>>()
			);
		}
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
	fn no_animation_when_done() {
		let (mut app, agent) = setup();
		app.world.entity_mut(agent).insert((
			_Dequeue {
				active: Some(Box::new(|| {
					let mut skill = mock_skill_without_default_setup_for([MockOption::Animate]);
					skill.expect_update_state().return_const(
						HashSet::<StateMeta<SkillState>>::from([StateMeta::Leaving(
							SkillState::AfterCast,
						)]),
					);
					skill
						.expect_animate()
						.return_const(Animate::Repeat(_AnimationKey::A));
					skill
				})),
			},
			Transform::default(),
		));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<Animate<_AnimationKey>>());
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
	fn no_facing_override_when_skill_ended() {
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
			Transform::from_xyz(-1., -2., -3.),
			OverrideFace(Face::Cursor),
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<OverrideFace>());
	}
}
