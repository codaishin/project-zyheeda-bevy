use super::GetExecution;
use crate::skills::{SkillCaster, SkillExecution, SkillSpawner, Target};
use behaviors::components::Projectile;
use bevy::{ecs::system::EntityCommands, prelude::SpatialBundle, transform::components::Transform};

impl<T: Send + Sync + 'static> GetExecution for Projectile<T> {
	fn execution() -> SkillExecution {
		SkillExecution {
			run_fn: Some(run_fn::<T>),
			stop_fn: None,
		}
	}
}

fn run_fn<T: Send + Sync + 'static>(
	agent: &mut EntityCommands,
	caster: &SkillCaster,
	spawner: &SkillSpawner,
	_: &Target,
) {
	let transform = Transform::from_translation(spawner.0.translation());
	agent.commands().spawn((
		Projectile::<T>::new(caster.0.forward(), 10.),
		SpatialBundle::from_transform(transform),
	));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::test_tools::run_lazy;
	use bevy::{
		app::{App, Update},
		math::{primitives::Direction3d, Ray3d, Vec3},
		render::view::{InheritedVisibility, ViewVisibility, Visibility},
		transform::components::GlobalTransform,
		utils::default,
	};
	use common::test_tools::utils::assert_eq_approx;

	#[test]
	fn spawn_projectile_with_agent_forward() {
		let mut app = App::new();
		let lazy = Projectile::<()>::execution();
		let forward = Vec3::new(8., 9., 10.);
		let caster = SkillCaster(Transform::default().looking_at(forward, Vec3::Y));
		let spawner = SkillSpawner(GlobalTransform::from_xyz(1., 2., 3.));
		let agent = app.world.spawn(()).id();

		app.add_systems(
			Update,
			run_lazy(lazy, agent, caster, spawner, Target::default()),
		);
		app.update();

		let projectile = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<Projectile<()>>());

		assert_eq_approx!(
			Some(forward.normalize()),
			projectile.map(|p| p.direction.into()),
			0.0001
		);
	}

	#[test]
	fn spawn_with_special_bundle() {
		let mut app = App::new();
		let lazy = Projectile::<()>::execution();
		let caster = SkillCaster::default();
		let spawner = SkillSpawner(GlobalTransform::from_xyz(1., 2., 3.));
		let select_info = Target {
			ray: Ray3d {
				origin: Vec3::ONE,
				direction: Direction3d::NEG_Y,
			},
			..default()
		};
		let agent = app.world.spawn(()).id();

		app.add_systems(Update, run_lazy(lazy, agent, caster, spawner, select_info));
		app.update();

		let projectile = app
			.world
			.iter_entities()
			.find(|e| e.contains::<Projectile<()>>())
			.unwrap();

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
		let lazy = Projectile::<()>::execution();
		let caster = SkillCaster::default();
		let spawner = SkillSpawner(GlobalTransform::from_xyz(1., 2., 3.));
		let select_info = Target {
			ray: Ray3d {
				origin: Vec3::ONE,
				direction: Direction3d::NEG_Y,
			},
			..default()
		};
		let agent = app.world.spawn(()).id();

		app.add_systems(Update, run_lazy(lazy, agent, caster, spawner, select_info));
		app.update();

		let projectile_transform = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<Transform>())
			.unwrap();

		assert_eq!(Vec3::new(1., 2., 3.), projectile_transform.translation)
	}
}
