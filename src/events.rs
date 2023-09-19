use bevy::prelude::*;

#[derive(Event, Clone, Copy)]
pub struct MoveEvent {
	pub target: Vec3,
}

#[derive(Event)]
pub struct MoveEnqueueEvent {
	pub target: Vec3,
}
