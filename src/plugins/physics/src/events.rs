use bevy::{
	ecs::{entity::Entity, event::Event},
	math::Ray3d,
};
use common::traits::handles_physics::TimeOfImpact;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Ray(pub Ray3d, pub TimeOfImpact);

#[derive(Event, Debug, PartialEq, Clone, Copy)]
pub struct RayEvent(pub Entity, pub Ray);

/// Signals intersections of a beam with the given entity
///
/// Must be reported each frame
#[derive(Event, Debug, PartialEq)]
pub(crate) struct BeamInteraction {
	pub(crate) beam: Entity,
	pub(crate) intersects: Entity,
}
