use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SimpleMovement {
	pub target: Option<Vec3>,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum MovementMode {
	#[default]
	Walk,
	Run,
}

#[derive(PartialEq, Debug)]
pub struct Walk;

#[derive(PartialEq, Debug)]
pub struct Run;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Idle;

#[derive(PartialEq, Debug)]
pub enum Behavior {
	SimpleMovement((SimpleMovement, MovementMode)),
	Idle(Idle),
}
