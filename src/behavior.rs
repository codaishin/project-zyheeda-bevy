use bevy::prelude::*;

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct SimpleMovement {
	pub target: Vec3,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum MovementMode {
	#[default]
	Walk,
	Run,
}

#[derive(Component, PartialEq, Debug)]
pub struct Walk;

#[derive(Component, PartialEq, Debug)]
pub struct Run;

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct Idle;

#[derive(PartialEq, Debug)]
pub enum BehaviorOld {
	SimpleMovement((SimpleMovement, MovementMode)),
	Idle(Idle),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Behavior {
	MoveTo(Vec3),
}
