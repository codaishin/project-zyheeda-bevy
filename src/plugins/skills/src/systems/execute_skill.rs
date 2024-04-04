use crate::{
	components::{SlotKey, SlotVisibility, Slots},
	skill::{SkillState, Spawner, Target},
	traits::{Execution, GetAnimation, GetSlots},
};
use behaviors::components::{Face, OverrideFace};
use bevy::{
	ecs::{
		component::Component,
		entity::Entity,
		system::{Commands, EntityCommands, Query, Res},
	},
	math::Ray3d,
	time::Time,
	transform::components::{GlobalTransform, Transform},
};
use common::{
	components::{Animate, Idle, Outdated},
	resources::{CamRay, ColliderInfo, MouseHover},
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
	mut commands: Commands,
	time: Res<Time<TTime>>,
	cam_ray: Res<CamRay>,
	mouse_hover: Res<MouseHover>,
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
			agent.try_insert(OverrideFace(Face::Cursor));
			agent.try_insert(SlotVisibility::Inherited(skill.slots()));
		}
		if states.contains(&StateMeta::In(SkillState::Aim)) {
			agent.try_insert(OverrideFace(Face::Cursor));
		}
		if states.contains(&StateMeta::Leaving(SkillState::PreCast)) {
			handle_active(
				agent,
				skill,
				agent_transform,
				transforms,
				slots,
				&cam_ray,
				&mouse_hover,
			);
		}

		if states.contains(&StateMeta::Leaving(SkillState::AfterCast)) {
			agent.try_insert(Idle);
			agent.try_insert(SlotVisibility::Hidden(skill.slots()));
			agent.remove::<(TSkill, OverrideFace, Animate<TAnimationKey>)>();
			skill.stop(agent);
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

fn get_target(
	transforms: &Query<&GlobalTransform>,
	ray: Ray3d,
	hover: &Option<ColliderInfo<Entity>>,
) -> Target {
	Target {
		ray,
		collision_info: hover.as_ref().and_then(|hover| {
			Some(ColliderInfo {
				collider: Outdated {
					entity: hover.collider,
					component: *transforms.get(hover.collider).ok()?,
				},
				root: hover.root.and_then(|entity| {
					transforms.get(entity).ok().map(|transform| Outdated {
						entity,
						component: *transform,
					})
				}),
			})
		}),
	}
}

fn handle_active<TSkill: Execution>(
	agent: &mut EntityCommands,
	skill: &mut TSkill,
	agent_transform: &Transform,
	transforms: &Query<&GlobalTransform>,
	slots: &Slots,
	cam_ray: &Res<CamRay>,
	mouse_hover: &Res<MouseHover>,
) {
	let Some(spawner) = get_spawner(slots, transforms) else {
		return;
	};
	let Some(ray) = cam_ray.0 else {
		return;
	};
	let target = get_target(transforms, ray, &mouse_hover.0);
	skill.run(agent, agent_transform, &spawner, &target);
}

fn get_spawner(slots: &Slots, transforms: &Query<&GlobalTransform>) -> Option<Spawner> {
	let spawner_slot = slots.0.get(&SlotKey::SkillSpawn)?;
	let spawner_transform = transforms.get(spawner_slot.entity).ok()?;
	Some(Spawner(*spawner_transform))
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Slot, SlotVisibility};
	use behaviors::components::{Face, OverrideFace};
	use bevy::{
		math::Ray3d,
		prelude::{App, Transform, Update, Vec3},
		time::{Real, Time},
	};
	use common::{
		components::{Outdated, Side},
		resources::{CamRay, ColliderInfo, MouseHover},
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
		fn run(
			&self,
			agent: &mut EntityCommands,
			agent_transform: &Transform,
			spawner: &Spawner,
			target: &Target,
		) {
			self.mock.run(agent, agent_transform, spawner, target)
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
			fn run<'a>(&self, agent: &mut EntityCommands<'a>, agent_transform: &Transform, spawner: &Spawner, target: &Target) {}
			fn stop<'a>(&self, agent: &mut EntityCommands<'a>) {}
		}
		impl GetAnimation<_AnimationKey> for _Skill {
			fn animate(&self) -> Animate<_AnimationKey> {}
		}
		impl GetSlots for _Skill {
			fn slots(&self) -> Vec<SlotKey> {}
		}
	}

	fn setup(skill_spawn_location: Vec3, agent_location: Vec3) -> (App, Entity) {
		let mut app = App::new_single_threaded([Update]);
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
		app.init_resource::<CamRay>();
		app.init_resource::<MouseHover>();
		app.update();
		app.add_systems(Update, execute_skill::<_AnimationKey, _Skill, Real>);

		(app, agent)
	}

	#[test]
	fn call_update_with_delta() {
		let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
			let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
		let skill_spawn_location = Vec3::new(1., 2., 3.);
		let agent_location = Vec3::new(3., 4., 5.);
		let (mut app, agent_entity) = setup(skill_spawn_location, agent_location);
		let mut skill =
			_Skill::without_default_setup_for([MockOption::BehaviorExecution(BehaviorOption::Run)]);
		let ray = Ray3d::new(Vec3::new(1., 2., 3.), Vec3::new(4., 5., 6.));
		let mouse_hover_transform = GlobalTransform::from_xyz(10., 10., 10.);
		let mouse_hover = app.world.spawn(mouse_hover_transform).id();
		let mouse_hover_root_transform = GlobalTransform::from_xyz(11., 11., 11.);
		let mouse_hover_root = app.world.spawn(mouse_hover_root_transform).id();
		app.insert_resource(CamRay(Some(ray)));
		app.insert_resource(MouseHover(Some(ColliderInfo {
			collider: mouse_hover,
			root: Some(mouse_hover_root),
		})));

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
			.withf(move |agent, agent_transform, skill_spawn, target| {
				assert_eq!(
					(
						agent_entity,
						&Transform::from_translation(agent_location),
						GlobalTransform::from_translation(skill_spawn_location),
						&Target {
							ray,
							collision_info: Some(ColliderInfo {
								collider: Outdated {
									entity: mouse_hover,
									component: mouse_hover_transform,
								},
								root: Some(Outdated {
									entity: mouse_hover_root,
									component: mouse_hover_root_transform,
								}),
							}),
						}
					),
					(agent.id(), agent_transform, skill_spawn.0, target)
				);
				true
			})
			.return_const(());

		app.world.entity_mut(agent_entity).insert(skill);

		app.update();
	}

	#[test]
	fn do_run_when_not_activating_this_frame() {
		let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::new(11., 12., 13.), Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::ZERO, Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::new(11., 12., 13.), Vec3::ZERO);
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
		let (mut app, agent) = setup(Vec3::new(11., 12., 13.), Vec3::ZERO);
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
