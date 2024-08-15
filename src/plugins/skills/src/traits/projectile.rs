use super::{GetStaticSkillBehavior, SkillBundleConfig, SpawnSkill};
use crate::{
	behaviors::{spawn_behavior::SpawnBehavior, Behavior, SkillCaster, SkillSpawner, Target},
	skills::{SkillBehavior, SkillBehaviors},
};
use behaviors::components::Projectile;
use bevy::{
	ecs::bundle::Bundle,
	prelude::SpatialBundle,
	transform::components::Transform,
	utils::default,
};

impl<T: Send + Sync + 'static> SkillBundleConfig for Projectile<T> {
	const STOPPABLE: bool = false;

	fn new_skill_bundle(caster: &SkillCaster, spawner: &SkillSpawner, _: &Target) -> impl Bundle {
		(
			Projectile::<T>::new(caster.1.forward(), 10.),
			SpatialBundle::from_transform(Transform::from(spawner.1)),
		)
	}
}

impl<T: Send + Sync + 'static> GetStaticSkillBehavior for Projectile<T> {
	fn behavior() -> SkillBehavior {
		SkillBehavior::OnActive(SkillBehaviors {
			contact: Behavior::new().with_spawn(SpawnBehavior::Fn(Projectile::<T>::spawn_skill)),
			..default()
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::skills::SelectInfo;
	use bevy::{
		app::App,
		ecs::entity::Entity,
		math::{Ray3d, Vec3},
		transform::components::GlobalTransform,
	};
	use common::{assert_bundle, assert_eq_approx};

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
			GlobalTransform::from(Transform::default().looking_at(forward, Vec3::Y)),
		);
		let spawner = SkillSpawner(Entity::from_raw(43), GlobalTransform::from_xyz(1., 2., 3.));

		let projectile = app
			.world_mut()
			.spawn(Projectile::<()>::new_skill_bundle(
				&caster,
				&spawner,
				&target(),
			))
			.id();
		let projectile = app.world().entity(projectile).get::<Projectile<()>>();

		assert_eq_approx!(
			Some(forward.normalize()),
			projectile.map(|p| p.direction.into()),
			0.0001
		);
	}

	#[test]
	fn spawn_with_special_bundle() {
		let mut app = App::new();
		let caster = SkillCaster(Entity::from_raw(42), GlobalTransform::default());
		let spawner = SkillSpawner(Entity::from_raw(43), GlobalTransform::from_xyz(1., 2., 3.));

		let projectile = app
			.world_mut()
			.spawn(Projectile::<()>::new_skill_bundle(
				&caster,
				&spawner,
				&target(),
			))
			.id();
		let projectile = app.world().entity(projectile);

		assert_bundle!(SpatialBundle, &app, projectile);
	}

	#[test]
	fn spawn_with_proper_location() {
		let mut app = App::new();
		let caster = SkillCaster(Entity::from_raw(42), GlobalTransform::default());
		let spawner = SkillSpawner(Entity::from_raw(43), GlobalTransform::from_xyz(1., 2., 3.));

		let projectile = app
			.world_mut()
			.spawn(Projectile::<()>::new_skill_bundle(
				&caster,
				&spawner,
				&target(),
			))
			.id();
		let projectile = app.world().entity(projectile).get::<Transform>();

		assert_eq!(
			Some(Vec3::new(1., 2., 3.)),
			projectile.map(|p| p.translation)
		)
	}
}
