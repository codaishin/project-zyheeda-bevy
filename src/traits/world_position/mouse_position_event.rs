#[cfg(test)]
mod set_world_position_tests;

use bevy::prelude::*;

use crate::events::MousePositionEvent;

use super::SetWorldPositionFromRay;

impl SetWorldPositionFromRay for MousePositionEvent {
	fn set_world_position(&mut self, ray_cast: Ray) {
		self.world_position = ray_cast
			.intersect_plane(self.collision_plane.origin, self.collision_plane.normal)
			.map(|distance| ray_cast.origin + ray_cast.direction * distance);
	}
}
