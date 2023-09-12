use bevy::prelude::Vec3;

use crate::events::{MouseEvent, Plane};
use crate::traits::new::New;

impl New for MouseEvent {
	fn new() -> Self {
		Self {
			collision_plane: Plane {
				origin: Vec3::ZERO,
				normal: Vec3::Y,
			},
			world_position: None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::Vec3;

	#[test]
	fn new_instance() {
		let event = MouseEvent::new();

		assert_eq!(
			(Vec3::ZERO, Vec3::Y, None),
			(
				event.collision_plane.origin,
				event.collision_plane.normal,
				event.world_position
			)
		)
	}
}
