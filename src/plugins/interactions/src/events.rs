use bevy::{
	ecs::{entity::Entity, event::Event},
	math::{Dir3, Ray3d, Vec3},
};
use common::traits::cast_ray::TimeOfImpact;

#[derive(Event, Debug, PartialEq, Clone, Copy)]
pub struct RayCastEvent {
	pub source: Entity,
	pub target: RayCastTarget,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct RayCastTarget {
	pub entity: Option<Entity>,
	pub ray: Ray3d,
	pub toi: TimeOfImpact,
}

impl Default for RayCastTarget {
	fn default() -> Self {
		Self {
			entity: None,
			ray: Ray3d {
				origin: Vec3::ZERO,
				direction: Dir3::NEG_Z,
			},
			toi: TimeOfImpact::default(),
		}
	}
}
