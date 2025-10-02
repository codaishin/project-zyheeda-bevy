use crate::{
	tools::speed::Speed,
	traits::accessors::get::{GetProperty, Property},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub trait HandlesMovementBehavior {
	type TMovement: Component + From<PathMotionSpec> + GetProperty<PathMotionSpec>;
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct PathMotionSpec(pub MotionSpec);

impl Property for PathMotionSpec {
	type TValue<'a> = MotionSpec;
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum MotionSpec {
	Direction { speed: Speed, direction: Dir3 },
	ToTarget { speed: Speed, target: Vec3 },
	Stop,
}
