use bevy::prelude::*;
use common::traits::handles_physics::TimeOfImpact;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Ray(pub Ray3d, pub TimeOfImpact);

#[derive(Message, Debug, PartialEq, Clone, Copy)]
pub struct RayEvent(pub Entity, pub Ray);

/// Signals intersections of a beam with the given entity
///
/// Must be reported each frame
#[derive(Message, Debug, PartialEq)]
pub(crate) struct BeamInteraction {
	pub(crate) beam: Entity,
	pub(crate) intersects: Entity,
}
