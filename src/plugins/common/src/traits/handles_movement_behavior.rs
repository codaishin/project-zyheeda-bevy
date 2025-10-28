use crate::{
	tools::{Units, speed::Speed},
	traits::accessors::get::{EntityContext, EntityContextMut, GetProperty, Property},
};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

pub trait HandlesMovementBehavior {
	type TReadMovement<'w, 's>: for<'c> EntityContext<Movement, TContext<'c>: GetProperty<PathMotionSpec>>;
	type TWriteMovement<'w, 's>: for<'c> EntityContextMut<Movement, TContext<'c>: SetMovement>;
}

pub trait SetMovement {
	fn set_movement(&mut self, spec: PathMotionSpec);
}

pub struct Movement;

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
