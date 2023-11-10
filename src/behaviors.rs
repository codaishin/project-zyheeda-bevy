pub mod move_to;

use bevy::prelude::*;

use crate::components::{Cast, Side};

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum MovementMode {
	#[default]
	Walk,
	Run,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Behavior {
	MoveTo(Vec3),
	ShootGun(Ray, Cast, Side),
}
