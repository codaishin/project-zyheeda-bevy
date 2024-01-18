use super::GetBehaviorMeta;
use crate::{
	behaviors::meta::{BehaviorMeta, Spawner},
	components::Projectile,
	tools::look_from_spawner,
};
use bevy::{
	ecs::system::EntityCommands,
	math::Ray,
	prelude::SpatialBundle,
	transform::components::Transform,
};

impl<T: Send + Sync + 'static> GetBehaviorMeta for Projectile<T> {
	fn behavior() -> BehaviorMeta {
		BehaviorMeta {
			run_fn: Some(run_fn::<T>),
			stop_fn: None,
			transform_fn: Some(look_from_spawner),
		}
	}
}

fn run_fn<T: Send + Sync + 'static>(
	agent: &mut EntityCommands,
	agent_transform: &Transform,
	spawner: &Spawner,
	_: &Ray,
) {
	let transform = Transform::from_translation(spawner.0.translation());
	agent.commands().spawn((
		Projectile::<T>::new(agent_transform.forward(), 10.),
		SpatialBundle::from_transform(transform),
	));
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{test_tools::assert_eq_approx, traits::behavior::test_tools::run_lazy};
	use bevy::{
		app::{App, Update},
		math::Vec3,
		render::view::{InheritedVisibility, ViewVisibility, Visibility},
		transform::components::GlobalTransform,
		utils::default,
	};

	#[test]
	fn spawn_projectile_with_agent_forward() {
		let mut app = App::new();
		let lazy = Projectile::<()>::behavior();
		let spawner = Spawner(GlobalTransform::from_xyz(1., 2., 3.));
		let forward = Vec3::new(8., 9., 10.);
		let agent = app.world.spawn(()).id();

		app.add_systems(
			Update,
			run_lazy(
				lazy,
				agent,
				Transform::default().looking_at(forward, Vec3::Y),
				spawner,
				Ray::default(),
			),
		);
		app.update();

		let projectile = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<Projectile<()>>());

		assert_eq_approx!(
			Some(forward.normalize()),
			projectile.map(|p| p.direction),
			0.0001
		);
	}

	#[test]
	fn spawn_with_special_bundle() {
		let mut app = App::new();
		let lazy = Projectile::<()>::behavior();
		let spawner = Spawner(GlobalTransform::from_xyz(1., 2., 3.));
		let ray = Ray {
			origin: Vec3::ONE,
			direction: Vec3::NEG_INFINITY,
		};
		let agent = app.world.spawn(()).id();

		app.add_systems(Update, run_lazy(lazy, agent, default(), spawner, ray));
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
		let lazy = Projectile::<()>::behavior();
		let spawner = Spawner(GlobalTransform::from_xyz(1., 2., 3.));
		let ray = Ray {
			origin: Vec3::ONE,
			direction: Vec3::NEG_INFINITY,
		};
		let agent = app.world.spawn(()).id();

		app.add_systems(Update, run_lazy(lazy, agent, default(), spawner, ray));
		app.update();

		let projectile_transform = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<Transform>())
			.unwrap();

		assert_eq!(Vec3::new(1., 2., 3.), projectile_transform.translation)
	}

	#[test]
	fn use_proper_transform_fn() {
		let lazy = Projectile::<()>::behavior();

		assert_eq!(
			Some(look_from_spawner as usize),
			lazy.transform_fn.map(|f| f as usize)
		);
	}
}
