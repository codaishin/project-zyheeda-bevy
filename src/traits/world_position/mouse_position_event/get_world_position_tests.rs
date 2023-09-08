use bevy::prelude::Vec3;

use crate::{
	events::{MousePositionEvent, Plane},
	traits::world_position::GetWorldPosition,
};

#[test]
fn get_world_position_none() {
	let event = MousePositionEvent {
		collision_plane: Plane {
			origin: Vec3::ZERO,
			normal: Vec3::Y,
		},
		world_position: None,
	};

	assert_eq!(None, event.get_world_position());
}

#[test]
fn get_world_position_some() {
	let event = MousePositionEvent {
		collision_plane: Plane {
			origin: Vec3::ZERO,
			normal: Vec3::Y,
		},
		world_position: Some(Vec3::ONE),
	};

	assert_eq!(Some(Vec3::ONE), event.get_world_position());
}
