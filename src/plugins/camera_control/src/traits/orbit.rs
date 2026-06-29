use bevy::prelude::*;

pub trait MoveArm {
	fn move_arm(&mut self, angles: Vec2Radians);
}

pub type Vec2Radians = Vec2;
