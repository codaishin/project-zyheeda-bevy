use bevy::{
	ecs::{entity::Entity, event::Event},
	math::Ray,
};
use common::traits::cast_ray::TimeOfImpact;

#[derive(Event, Debug, PartialEq, Clone, Copy)]
pub struct RayCastEvent {
	pub source: Entity,
	pub target: RayCastTarget,
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct RayCastTarget {
	pub entity: Option<Entity>,
	pub ray: Ray,
	pub toi: TimeOfImpact,
}
