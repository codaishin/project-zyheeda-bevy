use super::ToLazy;
use crate::components::{lazy::Lazy, Agent, Projectile, Spawner};
use bevy::{
	ecs::system::Commands,
	math::Ray,
	prelude::SpatialBundle,
	transform::components::Transform,
};

impl ToLazy for Projectile {
	fn to_lazy() -> Option<Lazy> {
		Some(Lazy::new(Some(run_fn), None))
	}
}

fn run_fn(commands: &mut Commands, _: Agent, spawner: Spawner, ray: Ray) {
	let transform = Transform::from_translation(spawner.0.translation());
	commands.spawn((
		Projectile {
			target_ray: ray,
			range: 10.,
		},
		SpatialBundle::from_transform(transform),
	));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::to_lazy::test_tools::run_lazy;
	use bevy::{
		app::{App, Update},
		math::Vec3,
		prelude::Entity,
		render::view::{InheritedVisibility, ViewVisibility, Visibility},
		transform::components::GlobalTransform,
	};

	const DEFAULT_AGENT: Agent = Agent(Entity::from_raw(42));

	#[test]
	fn spawn_projectile_with_ray() {
		let mut app = App::new();
		let lazy = Projectile::to_lazy();
		let spawner = Spawner(GlobalTransform::from_xyz(1., 2., 3.));
		let ray = Ray {
			origin: Vec3::ONE,
			direction: Vec3::NEG_INFINITY,
		};

		app.add_systems(Update, run_lazy(lazy, DEFAULT_AGENT, spawner, ray));
		app.update();

		let projectile = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<Projectile>());

		assert_eq!(Some(ray), projectile.map(|p| p.target_ray));
	}

	#[test]
	fn spawn_with_special_bundle() {
		let mut app = App::new();
		let lazy = Projectile::to_lazy();
		let spawner = Spawner(GlobalTransform::from_xyz(1., 2., 3.));
		let ray = Ray {
			origin: Vec3::ONE,
			direction: Vec3::NEG_INFINITY,
		};

		app.add_systems(Update, run_lazy(lazy, DEFAULT_AGENT, spawner, ray));
		app.update();

		let projectile = app
			.world
			.iter_entities()
			.find(|e| e.contains::<Projectile>())
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
		let lazy = Projectile::to_lazy();
		let spawner = Spawner(GlobalTransform::from_xyz(1., 2., 3.));
		let ray = Ray {
			origin: Vec3::ONE,
			direction: Vec3::NEG_INFINITY,
		};

		app.add_systems(Update, run_lazy(lazy, DEFAULT_AGENT, spawner, ray));
		app.update();

		let projectile_transform = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<Transform>())
			.unwrap();

		assert_eq!(Vec3::new(1., 2., 3.), projectile_transform.translation)
	}
}
