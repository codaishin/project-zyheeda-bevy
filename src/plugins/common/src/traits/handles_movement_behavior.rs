use crate::{
	tools::{Units, speed::Speed},
	traits::accessors::get::{GetProperty, Property},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub trait HandlesMovementBehavior {
	type TMovement: Component + From<PathMotionSpec> + GetProperty<PathMotionSpec>;
}

#[derive(Debug, PartialEq, Default, Clone, Copy, Serialize, Deserialize)]
pub struct PathMotionSpec {
	pub motion: MotionSpec,
	pub clearance_radius: Units,
}

impl PathMotionSpec {
	pub fn with_motion(mut self, motion: MotionSpec) -> Self {
		self.motion = motion;
		self
	}

	pub fn with_clearance_radius(mut self, radius: Units) -> Self {
		self.clearance_radius = radius;
		self
	}
}

impl Property for PathMotionSpec {
	type TValue<'a> = MotionSpec;
}

#[derive(Debug, PartialEq, Default, Clone, Copy, Serialize, Deserialize)]
pub enum MotionSpec {
	Direction {
		speed: Speed,
		direction: Dir3,
	},
	ToTarget {
		speed: Speed,
		target: Vec3,
	},
	#[default]
	Stop,
}
