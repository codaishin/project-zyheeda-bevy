use crate::behavior::{Behavior, MovementMode};
use bevy::prelude::*;
use std::{collections::VecDeque, marker::PhantomData};

#[derive(Component)]
pub struct CamOrbit {
	pub center: Vec3,
	pub distance: f32,
	pub sensitivity: f32,
}

/// Represents units per second.
/// Is clamped at minimum 0.
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug, Default)]
pub struct UnitsPerSecond(f32);

impl UnitsPerSecond {
	pub fn new(value: f32) -> Self {
		if value < 0. {
			Self(0.)
		} else {
			Self(value)
		}
	}

	pub fn to_f32(self) -> f32 {
		self.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn set_value() {
		let speed = UnitsPerSecond::new(42.);

		assert_eq!(42., speed.to_f32());
	}

	#[test]
	fn min_zero() {
		let speed = UnitsPerSecond::new(-42.);

		assert_eq!(0., speed.to_f32());
	}
}

#[derive(Component, Default)]
pub struct Player {
	pub walk_speed: UnitsPerSecond,
	pub run_speed: UnitsPerSecond,
	pub movement_mode: MovementMode,
}

#[derive(Component)]
pub struct Behaviors(pub Vec<Behavior>);

#[derive(Component, Default)]
pub struct Animator {
	pub animation_player_id: Option<Entity>,
}

#[derive(Component)]
pub struct Queue<T>(pub VecDeque<T>);

#[derive(Component)]
pub struct BusyExecuting<T> {
	phantom_data: PhantomData<T>,
}

impl<T> Default for BusyExecuting<T> {
	fn default() -> Self {
		Self {
			phantom_data: default(),
		}
	}
}
