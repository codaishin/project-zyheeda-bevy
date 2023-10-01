use crate::behaviors::SimpleMovement;
use bevy::prelude::*;

#[derive(Component)]
pub struct CamOrbit {
	pub center: Vec3,
	pub distance: f32,
	pub sensitivity: f32,
}

/// Represents units per second.
/// Is clamped at minimum 0.
#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
pub struct UnitsPerSecond(f32);

impl UnitsPerSecond {
	pub fn new(value: f32) -> Self {
		match value < 0. {
			true => Self(0.),
			false => Self(value),
		}
	}

	pub fn unpack(&self) -> f32 {
		self.0
	}
}

#[derive(Component)]
pub struct Player {
	pub movement_speed: UnitsPerSecond,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn set_value() {
		let speed = UnitsPerSecond::new(42.);

		assert_eq!(42., speed.unpack());
	}

	#[test]
	fn min_zero() {
		let speed = UnitsPerSecond::new(-42.);

		assert_eq!(0., speed.unpack());
	}
}

#[derive(Component)]
pub struct Behaviors(pub Vec<SimpleMovement>);

#[derive(Component)]
pub struct PlayerAnimator;
