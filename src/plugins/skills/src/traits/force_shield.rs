use super::{GetExecution, NewSkillBundle, RunSkill};
use crate::skills::{Run, SkillCaster, SkillExecution, SkillSpawner, Target};
use behaviors::components::ForceShield;
use bevy::{self, prelude::SpatialBundle, transform::components::Transform};

impl NewSkillBundle for ForceShield {
	type Bundle = (ForceShield, SpatialBundle);

	fn new_skill_bundle(caster: &SkillCaster, spawner: &SkillSpawner, _: &Target) -> Self::Bundle {
		(
			ForceShield {
				direction: caster.1.forward(),
			},
			SpatialBundle::from_transform(Transform::from(spawner.0)),
		)
	}
}

impl GetExecution for ForceShield {
	fn execution() -> SkillExecution {
		SkillExecution {
			run_fn: Run::OnAim(ForceShield::run_skill),
			execution_stop_on_skill_stop: true,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{skills::SelectInfo, test_tools::assert_spacial_bundle};
	use bevy::{
		app::App,
		ecs::entity::Entity,
		math::{Ray3d, Vec3},
		transform::components::{GlobalTransform, Transform},
	};
	use common::assert_eq_approx;

	fn target() -> Target {
		SelectInfo {
			ray: Ray3d::new(Vec3::ONE, Vec3::ONE),
			collision_info: None,
		}
	}

	#[test]
	fn spawn_with_forward_direction() {
		let mut app = App::new();
		let forward = Vec3::new(8., 9., 10.);
		let caster = SkillCaster(
			Entity::from_raw(42),
			Transform::default().looking_at(forward, Vec3::Y),
		);
		let spawner = SkillSpawner(GlobalTransform::from_xyz(1., 2., 3.));

		let force_shield = app
			.world
			.spawn(ForceShield::new_skill_bundle(&caster, &spawner, &target()))
			.id();
		let force_shield = app.world.entity(force_shield).get::<ForceShield>();

		assert_eq_approx!(
			Some(forward.normalize()),
			force_shield.map(|p| p.direction.into()),
			0.0001
		);
	}

	#[test]
	fn spawn_with_special_bundle() {
		let mut app = App::new();
		let caster = SkillCaster(Entity::from_raw(42), Transform::default());
		let spawner = SkillSpawner(GlobalTransform::from_xyz(1., 2., 3.));

		let force_shield = app
			.world
			.spawn(ForceShield::new_skill_bundle(&caster, &spawner, &target()))
			.id();
		let force_shield = app.world.entity(force_shield);

		assert_spacial_bundle!(force_shield);
	}

	#[test]
	fn spawn_with_proper_location() {
		let mut app = App::new();
		let caster = SkillCaster(Entity::from_raw(42), Transform::default());
		let spawner = SkillSpawner(GlobalTransform::from_xyz(1., 2., 3.));

		let force_shield = app
			.world
			.spawn(ForceShield::new_skill_bundle(&caster, &spawner, &target()))
			.id();
		let force_shield = app.world.entity(force_shield).get::<Transform>();

		assert_eq!(
			Some(Vec3::new(1., 2., 3.)),
			force_shield.map(|p| p.translation)
		)
	}
}
