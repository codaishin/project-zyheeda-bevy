use bevy::{
	ecs::{entity::Entity, event::Event},
	math::{Dir3, Ray3d, Vec3},
	utils::default,
};
use common::{components::ColliderRoot, traits::cast_ray::TimeOfImpact};

#[derive(Debug, PartialEq, Clone)]
pub struct Ray(pub Ray3d, pub TimeOfImpact);

#[derive(Debug, PartialEq, Clone)]
pub enum Collision {
	Started(ColliderRoot),
	Ended(ColliderRoot),
}

#[derive(Event, Debug, PartialEq, Clone)]
pub struct InteractionEvent<TOther = Collision>(pub ColliderRoot, pub TOther);

impl InteractionEvent<()> {
	pub fn of(entity: ColliderRoot) -> Self {
		Self(entity, ())
	}

	pub fn collision(self, other: Collision) -> InteractionEvent {
		InteractionEvent(self.0, other)
	}

	pub fn ray(self, ray: Ray3d, toi: TimeOfImpact) -> InteractionEvent<Ray> {
		InteractionEvent(self.0, Ray(ray, toi))
	}
}

#[derive(Debug, PartialEq, Clone)]
pub struct RayCastInfo {
	pub hits: Vec<(Entity, TimeOfImpact)>,
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
