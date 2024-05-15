use super::{GetExecution, RunSkill, SkillBundleConfig};
use crate::skills::{Run, SkillCaster, SkillExecution, SkillSpawner, Target};
use behaviors::components::Projectile;
use bevy::{prelude::SpatialBundle, transform::components::Transform};

impl<T: Send + Sync + 'static> SkillBundleConfig for Projectile<T> {
	type Bundle = (Projectile<T>, SpatialBundle);

	const STOPPABLE: bool = false;

	fn new_skill_bundle(caster: &SkillCaster, spawner: &SkillSpawner, _: &Target) -> Self::Bundle {
		(
			Projectile::<T>::new(caster.1.forward(), 10.),
			SpatialBundle::from_transform(Transform::from(spawner.1)),
		)
	}
}

impl<T: Send + Sync + 'static> GetExecution for Projectile<T> {
	fn execution() -> SkillExecution {
		SkillExecution {
			run_fn: Run::OnActive(Projectile::<T>::run_skill),
			execution_stop_on_skill_stop: false,
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
		transform::components::GlobalTransform,
	};
	use common::assert_eq_approx;

	fn target() -> Target {
		SelectInfo {
			ray: Ray3d::new(Vec3::ONE, Vec3::ONE),
			collision_info: None,
		}
	}

	#[test]
	fn spawn_with_agent_forward() {
		let mut app = App::new();
		let forward = Vec3::new(8., 9., 10.);
		let caster = SkillCaster(
			Entity::from_raw(42),
			Transform::default().looking_at(forward, Vec3::Y),
		);
		let spawner = SkillSpawner(Entity::from_raw(43), GlobalTransform::from_xyz(1., 2., 3.));

		let projectile = app
			.world
			.spawn(Projectile::<()>::new_skill_bundle(
				&caster,
				&spawner,
				&target(),
			))
			.id();
		let projectile = app.world.entity(projectile).get::<Projectile<()>>();

		assert_eq_approx!(
			Some(forward.normalize()),
			projectile.map(|p| p.direction.into()),
			0.0001
		);
	}

	#[test]
	fn spawn_with_special_bundle() {
		let mut app = App::new();
		let caster = SkillCaster(Entity::from_raw(42), Transform::default());
		let spawner = SkillSpawner(Entity::from_raw(43), GlobalTransform::from_xyz(1., 2., 3.));

		let projectile = app
			.world
			.spawn(Projectile::<()>::new_skill_bundle(
				&caster,
				&spawner,
				&target(),
			))
			.id();
		let projectile = app.world.entity(projectile);

		assert_spacial_bundle!(projectile);
	}

	#[test]
	fn spawn_with_proper_location() {
		let mut app = App::new();
		let caster = SkillCaster(Entity::from_raw(42), Transform::default());
		let spawner = SkillSpawner(Entity::from_raw(43), GlobalTransform::from_xyz(1., 2., 3.));

		let projectile = app
			.world
			.spawn(Projectile::<()>::new_skill_bundle(
				&caster,
				&spawner,
				&target(),
			))
			.id();
		let projectile = app.world.entity(projectile).get::<Transform>();

		assert_eq!(
			Some(Vec3::new(1., 2., 3.)),
			projectile.map(|p| p.translation)
		)
	}
}
