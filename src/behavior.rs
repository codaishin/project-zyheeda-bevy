use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum MovementMode {
	#[default]
	Walk,
	Run,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Behavior {
	MoveTo(Vec3),
}
