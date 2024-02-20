use bevy::{
	ecs::{entity::Entity, event::Event},
	math::Ray,
};
use common::traits::cast_ray::TimeOfImpact;

#[derive(Event, Debug, PartialEq)]
pub struct RayCastEvent {
	pub source: Entity,
	pub target: RayCastTarget,
}

#[derive(Debug, PartialEq)]
pub enum RayCastTarget {
	None {
		ray: Ray,
		max_toi: TimeOfImpact,
	},
	Some {
		target: Entity,
		ray: Ray,
		toi: TimeOfImpact,
	},
}
