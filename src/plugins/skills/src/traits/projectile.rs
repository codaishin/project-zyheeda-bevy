use super::{GetExecution, NewSkillBundle, RunSkillDetached};
use crate::skills::{SkillCaster, SkillExecution, SkillSpawner, Target};
use behaviors::components::Projectile;
use bevy::{ecs::bundle::Bundle, prelude::SpatialBundle, transform::components::Transform};

impl<T: Send + Sync + 'static> NewSkillBundle for Projectile<T> {
	fn new_bundle(caster: &SkillCaster, spawner: &SkillSpawner, _: &Target) -> impl Bundle {
		(
			Projectile::<T>::new(caster.0.forward(), 10.),
			SpatialBundle::from_transform(Transform::from_translation(spawner.0.translation())),
		)
	}
}

impl<T: Send + Sync + 'static> GetExecution for Projectile<T> {
	fn execution() -> SkillExecution {
		SkillExecution {
			run_fn: Some(Projectile::<T>::run_detached),
			stop_fn: None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::skills::SelectInfo;
	use bevy::{
		app::App,
		math::{Ray3d, Vec3},
		render::view::{InheritedVisibility, ViewVisibility, Visibility},
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
		let caster = SkillCaster(Transform::default().looking_at(forward, Vec3::Y));
		let spawner = SkillSpawner(GlobalTransform::from_xyz(1., 2., 3.));

		let projectile = app
			.world
			.spawn(Projectile::<()>::new_bundle(&caster, &spawner, &target()))
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
		let caster = SkillCaster::default();
		let spawner = SkillSpawner(GlobalTransform::from_xyz(1., 2., 3.));

		let projectile = app
			.world
			.spawn(Projectile::<()>::new_bundle(&caster, &spawner, &target()))
			.id();
		let projectile = app.world.entity(projectile);

		assert_eq!(
			(true, true, true, true, true),
			(
				projectile.contains::<Visibility>(),
				projectile.contains::<InheritedVisibility>(),
				projectile.contains::<ViewVisibility>(),
				projectile.contains::<Transform>(),
				projectile.contains::<GlobalTransform>(),
			)
		)
	}

	#[test]
	fn spawn_with_proper_location() {
		let mut app = App::new();
		let caster = SkillCaster::default();
		let spawner = SkillSpawner(GlobalTransform::from_xyz(1., 2., 3.));

		let projectile = app
			.world
			.spawn(Projectile::<()>::new_bundle(&caster, &spawner, &target()))
			.id();
		let projectile = app.world.entity(projectile).get::<Transform>();

		assert_eq!(
			Some(Vec3::new(1., 2., 3.)),
			projectile.map(|p| p.translation)
		)
	}
}
