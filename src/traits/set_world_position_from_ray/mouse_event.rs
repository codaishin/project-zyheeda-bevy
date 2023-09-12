use crate::events::MouseEvent;
use bevy::prelude::*;

use super::SetWorldPositionFromRay;

impl SetWorldPositionFromRay for MouseEvent {
	fn set_world_position(&mut self, ray_cast: Ray) {
		self.world_position = ray_cast
			.intersect_plane(self.collision_plane.origin, self.collision_plane.normal)
			.map(|distance| ray_cast.origin + ray_cast.direction * distance);
	}
}

#[cfg(test)]
mod tests {
	use bevy::prelude::{Ray, Vec3};

	use crate::events::Plane;
	use crate::{events::MouseEvent, traits::set_world_position_from_ray::SetWorldPositionFromRay};

	#[test]
	fn set_world_position_center() {
		let mut event = MouseEvent {
			collision_plane: Plane {
				origin: Vec3::ZERO,
				normal: Vec3::Y,
			},
			world_position: None,
		};
		event.set_world_position(Ray {
			origin: Vec3::Y,
			direction: -Vec3::Y,
		});

		assert_eq!(Vec3::ZERO, event.world_position.unwrap());
	}

	#[test]
	fn set_world_position_offset() {
		let mut event = MouseEvent {
			collision_plane: Plane {
				origin: Vec3::Y,
				normal: Vec3::Y,
			},
			world_position: None,
		};
		event.set_world_position(Ray {
			origin: Vec3::new(2., 3., 2.),
			direction: -Vec3::ONE,
		});

		assert_eq!(Vec3::new(0., 1., 0.), event.world_position.unwrap());
	}
}
