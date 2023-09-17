use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SimpleMovement {
	pub target: Option<Vec3>,
}
