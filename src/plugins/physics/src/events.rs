use crate::traits::{FromCollisionEvent, cast_ray::RayHit};
use bevy::{
	ecs::{entity::Entity, event::Event},
	math::{Dir3, Ray3d, Vec3},
	utils::default,
};
use bevy_rapier3d::prelude::CollisionEvent;
use common::traits::handles_physics::TimeOfImpact;
use zyheeda_core::prelude::Sorted;

#[derive(Debug, PartialEq, Clone)]
pub struct Ray(pub Ray3d, pub TimeOfImpact);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Collision {
	Started(Entity),
	Ended(Entity),
}

#[derive(Event, Debug, PartialEq, Clone, Copy)]
pub struct InteractionEvent<TOther = Collision>(pub Entity, pub TOther);

impl InteractionEvent<()> {
	pub fn of(entity: Entity) -> Self {
		Self(entity, ())
	}

	pub fn collision(self, other: Collision) -> InteractionEvent {
		InteractionEvent(self.0, other)
	}

	pub fn ray(self, ray: Ray3d, toi: TimeOfImpact) -> InteractionEvent<Ray> {
		InteractionEvent(self.0, Ray(ray, toi))
	}
}

impl FromCollisionEvent for InteractionEvent {
	fn from_collision<F>(event: &CollisionEvent, get_root: F) -> Self
	where
		F: Fn(Entity) -> Entity,
	{
		match event {
			CollisionEvent::Started(a, b, ..) => {
				InteractionEvent::of(get_root(*a)).collision(Collision::Started(get_root(*b)))
			}
			CollisionEvent::Stopped(a, b, ..) => {
				InteractionEvent::of(get_root(*a)).collision(Collision::Ended(get_root(*b)))
			}
		}
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct RayCastInfo {
	pub hits: Sorted<RayHit>,
	pub ray: Ray3d,
	pub max_toi: TimeOfImpact,
}

impl Default for RayCastInfo {
	fn default() -> Self {
		Self {
			hits: default(),
			ray: Ray3d {
				origin: Vec3::ZERO,
				direction: Dir3::NEG_Z,
			},
			max_toi: default(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy_rapier3d::rapier::prelude::CollisionEventFlags;

	#[test]
	fn interaction_event_from_collision_mapped_with_get_root_fn() {
		const ROOT: Entity = Entity::from_raw(66);

		fn get_root(_: Entity) -> Entity {
			ROOT
		}

		let collisions = [
			CollisionEvent::Started(
				Entity::from_raw(42),
				Entity::from_raw(11),
				CollisionEventFlags::empty(),
			),
			CollisionEvent::Stopped(
				Entity::from_raw(42),
				Entity::from_raw(11),
				CollisionEventFlags::empty(),
			),
		];

		let interactions = collisions.map(|c| InteractionEvent::from_collision(&c, get_root));

		assert_eq!(
			[
				InteractionEvent::of(ROOT).collision(Collision::Started(ROOT)),
				InteractionEvent::of(ROOT).collision(Collision::Ended(ROOT))
			],
			interactions
		);
	}

	#[test]
	fn interaction_event_from_collision_mapped_with_correct_entities() {
		fn get_root(entity: Entity) -> Entity {
			entity
		}

		let collisions = [
			CollisionEvent::Started(
				Entity::from_raw(1),
				Entity::from_raw(2),
				CollisionEventFlags::empty(),
			),
			CollisionEvent::Stopped(
				Entity::from_raw(3),
				Entity::from_raw(4),
				CollisionEventFlags::empty(),
			),
		];

		let interactions = collisions.map(|c| InteractionEvent::from_collision(&c, get_root));

		assert_eq!(
			[
				InteractionEvent::of(Entity::from_raw(1))
					.collision(Collision::Started(Entity::from_raw(2))),
				InteractionEvent::of(Entity::from_raw(3))
					.collision(Collision::Ended(Entity::from_raw(4)))
			],
			interactions
		);
	}
}
