use super::Behavior;
use crate::components::SimpleMovement;
use bevy::{ecs::system::EntityCommands, math::Ray, prelude::Vec3};

fn insert_fn(entity: &mut EntityCommands, ray: Ray) {
	let Some(length) = ray.intersect_plane(Vec3::ZERO, Vec3::Y) else {
		return;
	};
	entity.insert(SimpleMovement {
		target: ray.origin + ray.direction * length,
	});
}

pub fn movement() -> Behavior {
	Behavior { insert_fn }
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{components::SimpleMovement, test_tools::assert_eq_approx};
	use bevy::prelude::{App, Commands, Entity, Ray, Update, Vec3};

	fn apply_behavior(behavior: Behavior, entity: Entity, ray: Ray) -> impl FnMut(Commands) {
		move |mut commands| {
			let mut entity = commands.entity(entity);
			behavior.insert_into(&mut entity, ray);
		}
	}

	#[test]
	fn move_to_zero() {
		let mut app = App::new();
		let behavior = movement();
		let entity = app.world.spawn(()).id();
		let ray = Ray {
			origin: Vec3::Y,
			direction: Vec3::NEG_Y,
		};

		app.add_systems(Update, apply_behavior(behavior, entity, ray));
		app.update();

		let movement = app.world.entity(entity).get::<SimpleMovement>();

		assert_eq!(Some(Vec3::ZERO), movement.map(|m| m.target));
	}

	#[test]
	fn move_to_offset() {
		let mut app = App::new();
		let behavior = movement();
		let entity = app.world.spawn(()).id();
		let ray = Ray {
			origin: Vec3::ONE,
			direction: Vec3::NEG_Y,
		};

		app.add_systems(Update, apply_behavior(behavior, entity, ray));
		app.update();

		let movement = app.world.entity(entity).get::<SimpleMovement>().unwrap();

		assert_eq!(Vec3::new(1., 0., 1.), movement.target);
	}

	#[test]
	fn move_to_offset_2() {
		let mut app = App::new();
		let behavior = movement();
		let entity = app.world.spawn(()).id();
		let ray = Ray {
			origin: Vec3::new(0., 4., 0.),
			direction: Vec3::new(0., -4., 3.),
		};

		app.add_systems(Update, apply_behavior(behavior, entity, ray));
		app.update();

		let movement = app.world.entity(entity).get::<SimpleMovement>().unwrap();

		assert_eq_approx(Vec3::new(0., 0., 3.), movement.target, 0.000001);
	}
}
