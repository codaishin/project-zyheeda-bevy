use crate::{
	components::{Slot, SlotKey, Slots},
	skill::{SkillState, Spawner},
	traits::{Execution, GetAnimation, GetSlots},
};
use behaviors::components::{Face, OverrideFace};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		system::{Commands, EntityCommands, Query, Res},
	},
	render::view::Visibility,
	time::Time,
	transform::components::{GlobalTransform, Transform},
};
use common::{
	components::{Animate, Idle},
	traits::state_duration::{StateMeta, StateUpdate},
};
use std::{collections::HashSet, time::Duration};

type Skills<'a, TSkill> = (
	Entity,
	&'a mut Transform,
	&'a mut TSkill,
	&'a Slots,
	Option<&'a Idle>,
);

pub(crate) fn execute_skill<
	TAnimationKey: Copy + Clone + PartialEq + Send + Sync + 'static,
	TSkill: StateUpdate<SkillState> + Execution + GetAnimation<TAnimationKey> + GetSlots + Component,
	TTime: Send + Sync + Default + 'static,
>(
	time: Res<Time<TTime>>,
	mut commands: Commands,
	mut agents: Query<Skills<TSkill>>,
	transforms: Query<&GlobalTransform>,
) {
	let delta = time.delta();
	for (entity, mut agent_transform, mut skill, slots, idle) in &mut agents {
		let Some(agent) = &mut commands.get_entity(entity) else {
			continue;
		};

		let agent_transform = &mut agent_transform;
		let skill: &mut TSkill = &mut skill;
		let transforms = &transforms;

		let states = get_states(skill, &delta, idle);

		if states.contains(&StateMeta::First) {
			handle_new(agent, skill, slots);
		}
		if states.contains(&StateMeta::In(SkillState::Aim)) {
			agent.try_insert(OverrideFace(Face::Cursor));
		}
		if states.contains(&StateMeta::Leaving(SkillState::PreCast)) {
			handle_active(agent, agent_transform, skill, slots, transforms);
		}
		if states.contains(&StateMeta::Leaving(SkillState::AfterCast)) {
			handle_done(agent, skill, slots);
		} else {
			agent.try_insert(skill.animate());
		}
	}
}

fn set_slots_visibility<TSkill: GetSlots>(
	mut commands: Commands,
	skill: &TSkill,
	slots: &Slots,
	visibility: Visibility,
) {
	for slot in skill.slots().iter().filter_map(|s| slots.0.get(s)) {
		set_slot_visibility(&mut commands, slot, visibility);
	}
}

fn set_slot_visibility(commands: &mut Commands, slot: &Slot, visibility: Visibility) {
	let Some(mut entity) = commands.get_entity(slot.entity) else {
		return;
	};
	entity.try_insert(visibility);
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

fn handle_new<TSkill: GetSlots>(agent: &mut EntityCommands, skill: &mut TSkill, slots: &Slots) {
	agent.try_insert(OverrideFace(Face::Cursor));
	set_slots_visibility(agent.commands(), skill, slots, Visibility::Inherited);
}

fn handle_active<TSkill: Execution>(
	agent: &mut EntityCommands,
	agent_transform: &Transform,
	skill: &mut TSkill,
	slots: &Slots,
	transforms: &Query<&GlobalTransform>,
) {
	let Some(spawner) = get_spawner(slots, transforms) else {
		return;
	};
	skill.run(agent, agent_transform, &spawner);
}

fn handle_done<
	TAnimationKey: Copy + Clone + Send + Sync + PartialEq + 'static,
	TSkill: Execution + GetAnimation<TAnimationKey> + GetSlots + Component,
>(
	agent: &mut EntityCommands,
	skill: &mut TSkill,
	slots: &Slots,
) {
	agent.try_insert(Idle);
	agent.remove::<(TSkill, OverrideFace, Animate<TAnimationKey>)>();
	skill.stop(agent);
	set_slots_visibility(agent.commands(), skill, slots, Visibility::Hidden);
}

fn get_spawner(slots: &Slots, transforms: &Query<&GlobalTransform>) -> Option<Spawner> {
	let spawner_slot = slots.0.get(&SlotKey::SkillSpawn)?;
	let spawner_transform = transforms.get(spawner_slot.entity).ok()?;
	Some(Spawner(*spawner_transform))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Slot;
	use behaviors::components::{Face, OverrideFace};
	use bevy::{
		ecs::component::Component,
		prelude::{App, Transform, Update, Vec3},
		render::view::Visibility,
		time::{Real, Time},
	};
	use common::{components::Side, test_tools::utils::TickTime};
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
				mock.expect_run().return_const(());
			}
			if !no_setup.contains(&MockOption::BehaviorExecution(BehaviorOption::Stop)) {
				mock.expect_stop().return_const(());
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
		fn run(&self, agent: &mut EntityCommands, agent_transform: &Transform, spawner: &Spawner) {
			self.mock.run(agent, agent_transform, spawner)
		}

		fn stop(&self, agent: &mut EntityCommands) {
			self.mock.stop(agent)
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
			fn run<'a>(&self, agent: &mut EntityCommands<'a>, agent_transform: &Transform, spawner: &Spawner) {}
			fn stop<'a>(&self, agent: &mut EntityCommands<'a>) {}
		}
		impl GetAnimation<_AnimationKey> for _Skill {
			fn animate(&self) -> Animate<_AnimationKey> {}
		}
		impl GetSlots for _Skill {
			fn slots(&self) -> Vec<SlotKey> {}
		}
	}

	fn setup_app(skill_spawn_location: Vec3, agent_location: Vec3) -> (App, Entity) {
		let mut app = App::new();
		let mut time = Time::<Real>::default();

		let skill_spawner = app
			.world
			.spawn(GlobalTransform::from_translation(skill_spawn_location))
			.id();

		let main_hand_slot = app.world.spawn_empty().id();
		let off_hand_slot = app.world.spawn_empty().id();
		let agent = app
			.world
			.spawn((
				Slots(
					[
						(
							SlotKey::SkillSpawn,
							Slot {
								entity: skill_spawner,
								item: None,
								combo_skill: None,
							},
						),
						(
							SlotKey::Hand(Side::Main),
							Slot {
								entity: main_hand_slot,
								item: None,
								combo_skill: None,
							},
						),
						(
							SlotKey::Hand(Side::Off),
							Slot {
								entity: off_hand_slot,
								item: None,
								combo_skill: None,
							},
						),
					]
					.into(),
				),
				Transform::from_translation(agent_location),
			))
			.id();

		time.update();
		app.insert_resource(time);
		app.update();
		app.add_systems(Update, execute_skill::<_AnimationKey, _Skill, Real>);

		(app, agent)
	}

	#[test]
	fn call_update_with_delta() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
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
			let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
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
		let slots = agent.get::<Slots>().unwrap();
		let slot = slots
			.0
			.get(&SlotKey::Hand(Side::Main))
			.map(|s| app.world.entity(s.entity))
			.unwrap();

		assert_eq!(Some(&Visibility::Inherited), slot.get::<Visibility>());
	}

	#[test]
	fn set_multiple_slots_visible_on_first() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
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
		let slots = agent.get::<Slots>().unwrap();
		let main = slots
			.0
			.get(&SlotKey::Hand(Side::Main))
			.map(|s| app.world.entity(s.entity))
			.unwrap();
		let off = slots
			.0
			.get(&SlotKey::Hand(Side::Off))
			.map(|s| app.world.entity(s.entity))
			.unwrap();

		assert_eq!(
			(Some(&Visibility::Inherited), Some(&Visibility::Inherited)),
			(main.get::<Visibility>(), off.get::<Visibility>())
		);
	}

	#[test]
	fn hide_slot_when_done() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
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
		let slots = agent.get::<Slots>().unwrap();
		let slot = slots
			.0
			.get(&SlotKey::Hand(Side::Off))
			.map(|s| app.world.entity(s.entity))
			.unwrap();

		assert_eq!(Some(&Visibility::Hidden), slot.get::<Visibility>());
	}

	#[test]
	fn hide_multiple_slots_when_done() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
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
		let slots = agent.get::<Slots>().unwrap();
		let main = slots
			.0
			.get(&SlotKey::Hand(Side::Main))
			.map(|s| app.world.entity(s.entity))
			.unwrap();
		let off = slots
			.0
			.get(&SlotKey::Hand(Side::Off))
			.map(|s| app.world.entity(s.entity))
			.unwrap();

		assert_eq!(
			(Some(&Visibility::Hidden), Some(&Visibility::Hidden)),
			(main.get::<Visibility>(), off.get::<Visibility>())
		);
	}

	#[test]
	fn no_animation_when_done() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup_app(Vec3::new(1., 2., 3.), Vec3::new(3., 4., 5.));
		let mut skill =
			_Skill::without_default_setup_for([MockOption::BehaviorExecution(BehaviorOption::Run)]);

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([
				StateMeta::Leaving(SkillState::PreCast),
			]));
		skill
			.mock
			.expect_run()
			.times(1)
			.withf(move |a, a_t, s| {
				a.id() == agent
					&& *a_t == Transform::from_xyz(3., 4., 5.)
					&& s.0 == GlobalTransform::from_xyz(1., 2., 3.)
			})
			.return_const(());

		app.world.entity_mut(agent).insert(skill);

		app.update();
	}

	#[test]
	fn do_run_when_not_activating_this_frame() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill =
			_Skill::without_default_setup_for([MockOption::BehaviorExecution(BehaviorOption::Run)]);

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
				SkillState::Active,
			)]));
		skill.mock.expect_run().times(0).return_const(());

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();
	}

	#[test]
	fn stop() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([MockOption::BehaviorExecution(
			BehaviorOption::Stop,
		)]);

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([
				StateMeta::Leaving(SkillState::AfterCast),
			]));
		skill
			.mock
			.expect_stop()
			.times(1)
			.withf(move |a| a.id() == agent)
			.return_const(());

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();
	}

	#[test]
	fn do_not_stop_when_not_done() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
		let mut skill = _Skill::without_default_setup_for([MockOption::BehaviorExecution(
			BehaviorOption::Stop,
		)]);

		skill
			.mock
			.expect_update_state()
			.return_const(HashSet::<StateMeta<SkillState>>::from([StateMeta::In(
				SkillState::Active,
			)]));
		skill.mock.expect_stop().times(0).return_const(());

		app.world
			.entity_mut(agent)
			.insert((skill, Transform::default()));

		app.update();
	}

	#[test]
	fn apply_facing() {
		let (mut app, agent) = setup_app(Vec3::new(11., 12., 13.), Vec3::ZERO);
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
	fn do_not_apply_facing_when_not_new() {
		let (mut app, agent) = setup_app(Vec3::ZERO, Vec3::ZERO);
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
	fn apply_transform_when_aiming() {
		let (mut app, agent) = setup_app(Vec3::new(11., 12., 13.), Vec3::ZERO);
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
	fn no_transform_when_skill_ended() {
		let (mut app, agent) = setup_app(Vec3::new(11., 12., 13.), Vec3::ZERO);
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
