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
