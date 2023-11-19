use super::Behavior;
use crate::components::{
	marker::{HandGun, Marker, Right, Shoot},
	Agent,
	Cast,
	Projectile,
	Skill,
	Spawner,
};
use bevy::{
	ecs::system::{Commands, EntityCommands},
	math::Ray,
	prelude::SpatialBundle,
	transform::components::Transform,
};
use std::time::Duration;

fn spawn_projectile(commands: &mut Commands, _: Agent, spawner: Spawner, ray: Ray) {
	let transform = Transform::from_translation(spawner.0.translation());
	commands.spawn((Projectile { ray }, SpatialBundle::from_transform(transform)));
}

fn insert_fn(entity: &mut EntityCommands, ray: Ray) {
	entity.insert(Skill {
		ray,
		cast: Cast {
			pre: Duration::from_millis(300),
			after: Duration::from_millis(100),
		},
		marker_commands: Marker::<(Shoot, HandGun, Right)>::commands(),
		spawn_behavior: Some(spawn_projectile),
	});
}

pub fn shoot_gun() -> Behavior {
	Behavior { insert_fn }
}

#[cfg(test)]
mod test_spawn_projectile {
	use super::*;
	use bevy::{
		app::{App, Update},
		math::Vec3,
		prelude::Entity,
		render::view::{InheritedVisibility, ViewVisibility, Visibility},
		transform::components::GlobalTransform,
	};

	fn spawn_projectile_system(spawner: Spawner, ray: Ray) -> impl Fn(Commands) {
		move |mut commands: Commands| {
			spawn_projectile(&mut commands, Agent(Entity::from_raw(42)), spawner, ray);
		}
	}

	#[test]
	fn spawn_projectile_with_ray() {
		let mut app = App::new();
		let spawner = Spawner(GlobalTransform::from_xyz(1., 2., 3.));
		let ray = Ray {
			origin: Vec3::ONE,
			direction: Vec3::NEG_INFINITY,
		};

		app.add_systems(Update, spawn_projectile_system(spawner, ray));
		app.update();

		let projectile = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<Projectile>());

		assert_eq!(Some(ray), projectile.map(|p| p.ray));
	}

	#[test]
	fn spawn_with_special_bundle() {
		let mut app = App::new();
		let spawner = Spawner(GlobalTransform::from_xyz(1., 2., 3.));
		let ray = Ray {
			origin: Vec3::ONE,
			direction: Vec3::NEG_INFINITY,
		};

		app.add_systems(Update, spawn_projectile_system(spawner, ray));
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
		let spawner = Spawner(GlobalTransform::from_xyz(1., 2., 3.));
		let ray = Ray {
			origin: Vec3::ONE,
			direction: Vec3::NEG_INFINITY,
		};

		app.add_systems(Update, spawn_projectile_system(spawner, ray));
		app.update();

		let projectile_transform = app
			.world
			.iter_entities()
			.find_map(|e| e.get::<Transform>())
			.unwrap();

		assert_eq!(Vec3::new(1., 2., 3.), projectile_transform.translation)
	}
}

#[cfg(test)]
mod tests_shoot_gun {
	use super::*;
	use crate::components::Skill;
	use bevy::prelude::{App, Commands, Entity, Ray, Update, Vec3};

	fn apply_behavior(behavior: Behavior, entity: Entity, ray: Ray) -> impl FnMut(Commands) {
		move |mut commands| {
			let mut entity = commands.entity(entity);
			behavior.insert_into(&mut entity, ray);
		}
	}

	#[test]
	fn add_skill() {
		let mut app = App::new();
		let behavior = shoot_gun();
		let entity = app.world.spawn(()).id();
		let ray = Ray {
			origin: Vec3::Y,
			direction: Vec3::NEG_Y,
		};

		app.add_systems(Update, apply_behavior(behavior, entity, ray));
		app.update();

		let skill = app.world.entity(entity).get::<Skill>();

		assert_eq!(
			Some(&Skill {
				ray,
				cast: Cast {
					pre: Duration::from_millis(300),
					after: Duration::from_millis(100)
				},
				marker_commands: Marker::<(Shoot, HandGun, Right)>::commands(),
				spawn_behavior: Some(spawn_projectile),
			}),
			skill
		);
	}
}
