use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SimpleMovement {
	pub target: Option<Vec3>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum MovementMode {
	Walk,
}

#[derive(PartialEq, Debug)]
pub struct Walk;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Idle;

#[derive(PartialEq, Debug)]
pub enum Behavior {
	SimpleMovement((SimpleMovement, MovementMode)),
	Idle(Idle),
}
