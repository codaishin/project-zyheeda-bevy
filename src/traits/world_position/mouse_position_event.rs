#[cfg(test)]
mod get_world_position_tests;

#[cfg(test)]
mod set_world_position_tests;

use crate::events::MousePositionEvent;
use bevy::prelude::*;

use super::{GetWorldPosition, SetWorldPositionFromRay};

impl SetWorldPositionFromRay for MousePositionEvent {
	fn set_world_position(&mut self, ray_cast: Ray) {
		self.world_position = ray_cast
			.intersect_plane(self.collision_plane.origin, self.collision_plane.normal)
			.map(|distance| ray_cast.origin + ray_cast.direction * distance);
	}
}

impl GetWorldPosition for MousePositionEvent {
	fn get_world_position(&self) -> Option<Vec3> {
		self.world_position
	}
}
