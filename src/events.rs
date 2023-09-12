use bevy::prelude::*;

pub struct Plane {
	pub origin: Vec3,
	pub normal: Vec3,
}

#[derive(Event)]
pub struct MouseEvent {
	pub collision_plane: Plane,
	pub world_position: Option<Vec3>,
}

impl MouseEvent {
	pub fn new() -> Self {
		Self {
			collision_plane: Plane {
				origin: Vec3::ZERO,
				normal: Vec3::Y,
			},
			world_position: None,
		}
	}
}
