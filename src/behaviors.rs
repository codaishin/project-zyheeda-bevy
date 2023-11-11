use bevy::prelude::*;

use crate::components::{Cast, Side};

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum MovementMode {
	#[default]
	Walk,
	Run,
}

#[derive(Debug, Clone, Copy)]
pub enum ItemBehavior {
	Move,
	ShootGun(Cast),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum PlayerBehavior {
	MoveTo(Vec3),
	ShootGun(Ray, Cast, Side),
}
