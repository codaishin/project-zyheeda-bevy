use bevy::prelude::*;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SimpleMovement {
	pub target: Option<Vec3>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Idle;

#[derive(PartialEq, Debug)]
pub enum Behavior {
	SimpleMovement(SimpleMovement),
	Idle(Idle),
}
