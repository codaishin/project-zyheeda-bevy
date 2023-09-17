use bevy::prelude::*;

#[derive(Event)]
pub struct MoveEvent {
	pub target: Vec3,
}
