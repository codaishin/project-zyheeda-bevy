use crate::{
	tools::speed::Speed,
	traits::accessors::get::{GetProperty, Property},
};
use bevy::prelude::*;

pub trait HandlesMovementBehavior {
	type TMovement: Component + From<MovementPath> + GetProperty<MovementPath>;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MovementPath {
	Direction { speed: Speed, direction: Dir3 },
	ToTarget { speed: Speed, target: Vec3 },
	Stop,
}

impl Property for MovementPath {
	type TValue<'a> = Self;
}
