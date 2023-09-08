use bevy::prelude::{Ray, Vec3};

use crate::events::Plane;
use crate::{events::MousePositionEvent, traits::world_position::SetWorldPositionFromRay};

#[test]
fn set_world_position_center() {
	let mut event = MousePositionEvent {
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
	let mut event = MousePositionEvent {
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
