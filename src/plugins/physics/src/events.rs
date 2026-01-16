use crate::traits::cast_ray::RayHit;
use bevy::{
	ecs::{entity::Entity, event::Event},
	math::{Dir3, Ray3d, Vec3},
	utils::default,
};
use common::traits::handles_physics::TimeOfImpact;
use zyheeda_core::prelude::Sorted;

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
