use bevy::{
	ecs::{entity::Entity, event::Event},
	math::{Dir3, Ray3d, Vec3},
	utils::default,
};
use common::traits::cast_ray::TimeOfImpact;

#[derive(Event, Debug, PartialEq, Clone)]
pub struct RayCastEvent {
	pub source: Entity,
	pub info: RayCastInfo,
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
