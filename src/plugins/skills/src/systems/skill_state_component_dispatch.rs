use crate::{
	components::{SkillExecution, SlotVisibility},
	skill::SkillState,
	traits::{Execution, GetAnimation, GetSlots},
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
use common::{
	components::{Animate, Idle},
	traits::state_duration::{StateMeta, StateUpdate},
};
use std::{collections::HashSet, time::Duration};

type Skills<'a, TSkill> = (Entity, &'a mut TSkill, Option<&'a Idle>);

pub(crate) fn skill_state_component_dispatch<
	TAnimationKey: Copy + Clone + PartialEq + Send + Sync + 'static,
	TSkillTrack: StateUpdate<SkillState> + Execution + GetAnimation<TAnimationKey> + GetSlots + Component,
	TTime: Send + Sync + Default + 'static,
>(
	mut commands: Commands,
	time: Res<Time<TTime>>,
	mut agents: Query<Skills<TSkillTrack>>,
) {
	let delta = time.delta();
	for (entity, mut skill, idle) in &mut agents {
		let Some(agent) = &mut commands.get_entity(entity) else {
			continue;
		};

		let skill: &mut TSkillTrack = &mut skill;
		let states = get_states(skill, &delta, idle);

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
			agent.try_insert(Idle);
			agent.try_insert(SlotVisibility::Hidden(skill.slots()));
			agent.remove::<(TSkillTrack, OverrideFace, Animate<TAnimationKey>)>();
			insert_skill_execution_stop(agent, skill);
		} else {
			agent.try_insert(skill.animate());
		}
	}
}

fn get_states<TSkill: StateUpdate<SkillState>>(
	skill: &mut TSkill,
	delta: &Duration,
	wait_next: Option<&Idle>,
) -> HashSet<StateMeta<SkillState>> {
	if wait_next.is_some() {
		return [StateMeta::Leaving(SkillState::AfterCast)].into();
	}
	skill.update_state(*delta)
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
		components::Side,
		test_tools::utils::{SingleThreadedApp, TickTime},
	};
	use mockall::{mock, predicate::eq};
	use std::time::Duration;

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

	#[derive(Component)]
	struct _Skill {
		pub mock: Mock_Skill,
	}

	impl _Skill {
		pub fn without_default_setup_for<const N: usize>(no_setup: [MockOption; N]) -> Self {
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

			Self { mock }
		}
	}

	impl StateUpdate<SkillState> for _Skill {
		fn update_state(&mut self, delta: Duration) -> HashSet<StateMeta<SkillState>> {
			self.mock.update_state(delta)
		}
	}

	impl Execution for _Skill {
		fn get_start(&self) -> Option<StartBehaviorFn> {
			self.mock.get_start()
		}

		fn get_stop(&self) -> Option<StopBehaviorFn> {
			self.mock.get_stop()
		}
	}

	impl GetAnimation<_AnimationKey> for _Skill {
		fn animate(&self) -> Animate<_AnimationKey> {
			self.mock.animate()
		}
	}

	impl GetSlots for _Skill {
		fn slots(&self) -> Vec<SlotKey> {
			self.mock.slots()
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
		let mut app = App::new_single_threaded([Update]);
		let mut time = Time::<Real>::default();
		let agent = app.world.spawn(()).id();

		time.update();
		app.insert_resource(time);
		app.update();
		app.add_systems(
			Update,
			skill_state_component_dispatch::<_AnimationKey, _Skill, Real>,
		);

		(app, agent)
	}

	#[test]
	fn call_update_with_delta() {
		let (mut app, agent) = setup();
		let mut skill = _Skill::without_default_setup_for([]);

		skill
			.mock
			.expect_update_state()
			.times(1)
			.with(eq(Duration::from_millis(100)))
			.return_const(HashSet::<StateMeta<SkillState>>::default());
		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

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
			let mut skill = _Skill::without_default_setup_for([MockOption::Animate]);
			skill
				.mock
				.expect_update_state()
				.return_const(HashSet::<StateMeta<SkillState>>::from([state]));
			skill
				.mock
				.expect_animate()
				.return_const(Animate::Repeat(_AnimationKey::A));

			app.world
				.entity_mut(agent)
				.insert((skill, Transform::default()));
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
		let mut skill = _Skill::without_default_setup_for([MockOption::Slot]);
		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::First]));
		skill
			.mock
			.expect_slots()
			.return_const(vec![SlotKey::Hand(Side::Main)]);

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));
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
		let mut skill = _Skill::without_default_setup_for([MockOption::Slot]);
		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::First]));
		skill
			.mock
			.expect_slots()
			.return_const(vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)]);

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));
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
		let mut skill = _Skill::without_default_setup_for([MockOption::Slot]);
		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([
				StateMeta::Leaving(SkillState::AfterCast),
			]));
		skill
			.mock
			.expect_slots()
			.return_const(vec![SlotKey::Hand(Side::Off)]);

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

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
		let mut skill = _Skill::without_default_setup_for([MockOption::Slot]);
		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([
				StateMeta::Leaving(SkillState::AfterCast),
			]));
		skill
			.mock
			.expect_slots()
			.return_const(vec![SlotKey::Hand(Side::Main), SlotKey::Hand(Side::Off)]);

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));
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
		let mut skill = _Skill::without_default_setup_for([MockOption::Animate]);
		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([
				StateMeta::Leaving(SkillState::AfterCast),
			]));
		skill
			.mock
			.expect_animate()
			.return_const(Animate::Repeat(_AnimationKey::A));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));
		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<Animate<_AnimationKey>>());
	}

	#[test]
	fn remove_skill() {
		let (mut app, agent) = setup();
		let mut skill = _Skill::without_default_setup_for([]);

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([
				StateMeta::Leaving(SkillState::AfterCast),
			]));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<_Skill>());
	}

	#[test]
	fn do_not_remove_skill_when_not_done() {
		let (mut app, agent) = setup();
		let mut skill = _Skill::without_default_setup_for([]);

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
				SkillState::AfterCast,
			)]));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<_Skill>());
	}

	#[test]
	fn add_idle() {
		let (mut app, agent) = setup();
		let mut skill = _Skill::without_default_setup_for([]);

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([
				StateMeta::Leaving(SkillState::AfterCast),
			]));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();

		let agent = app.world.entity(agent);

		assert!(agent.contains::<Idle>());
	}

	#[test]
	fn do_not_add_idle_when_not_done() {
		let (mut app, agent) = setup();
		let mut skill = _Skill::without_default_setup_for([]);

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
				SkillState::AfterCast,
			)]));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();

		let agent = app.world.entity(agent);

		assert!(!agent.contains::<Idle>());
	}

	#[test]
	fn remove_all_related_components_when_idle_present() {
		let (mut app, agent) = setup();
		let mut skill = _Skill::without_default_setup_for([MockOption::Animate]);

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::default());
		skill
			.mock
			.expect_animate()
			.never()
			.return_const(Animate::Repeat(_AnimationKey::A));

		app.world.entity_mut(agent).insert((
			skill,
			Transform::default(),
			Animate::Repeat(_AnimationKey::A),
			Idle,
		));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(
			(false, false),
			(
				agent.contains::<_Skill>(),
				agent.contains::<Animate<_AnimationKey>>()
			)
		);
	}

	#[test]
	fn run() {
		let (mut app, agent) = setup();
		let mut skill =
			_Skill::without_default_setup_for([MockOption::BehaviorExecution(BehaviorOption::Run)]);

		fn start_behavior(_: &mut EntityCommands, _: &Transform, _: &Spawner, _: &Target) {}

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([
				StateMeta::Leaving(SkillState::PreCast),
			]));
		skill
			.mock
			.expect_get_start()
			.returning(|| Some(start_behavior));

		app.world.entity_mut(agent).insert(skill);

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
		let mut skill =
			_Skill::without_default_setup_for([MockOption::BehaviorExecution(BehaviorOption::Run)]);

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
				SkillState::Active,
			)]));
		skill.mock.expect_get_start().never().return_const(None);

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();
	}

	#[test]
	fn stop() {
		let (mut app, agent) = setup();
		let mut skill = _Skill::without_default_setup_for([MockOption::BehaviorExecution(
			BehaviorOption::Stop,
		)]);

		fn stop_fn(_: &mut EntityCommands) {}

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([
				StateMeta::Leaving(SkillState::AfterCast),
			]));
		skill.mock.expect_get_stop().returning(|| Some(stop_fn));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

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
		let mut skill = _Skill::without_default_setup_for([MockOption::BehaviorExecution(
			BehaviorOption::Stop,
		)]);

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
				SkillState::Active,
			)]));
		skill.mock.expect_get_stop().never().return_const(None);

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();
	}

	#[test]
	fn apply_facing() {
		let (mut app, agent) = setup();
		let mut skill = _Skill::without_default_setup_for([]);

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::First]));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

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
		let mut skill = _Skill::without_default_setup_for([]);

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
				SkillState::Active,
			)]));

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<OverrideFace>());
	}

	#[test]
	fn apply_apply_facing_override_when_aiming() {
		let (mut app, agent) = setup();
		let mut skill = _Skill::without_default_setup_for([]);

		let transform = Transform::from_xyz(-1., -2., -3.);

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
				SkillState::Aim,
			)]));

		app.world.entity_mut(agent).insert((skill, transform));

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
		let mut skill = _Skill::without_default_setup_for([]);

		let transform = Transform::from_xyz(-1., -2., -3.);

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([
				StateMeta::Leaving(SkillState::AfterCast),
			]));

		app.world
			.entity_mut(agent)
			.insert((skill, transform, OverrideFace(Face::Cursor)));

		app.update();

		let agent = app.world.entity(agent);

		assert_eq!(None, agent.get::<OverrideFace>());
	}
}
