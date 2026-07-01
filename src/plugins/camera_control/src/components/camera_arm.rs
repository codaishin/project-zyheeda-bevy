use crate::traits::orbit::{MoveArm, Vec2Radians};
use bevy::prelude::*;
use common::{self, tools::Units};
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
#[savable_component(id = "camera arm")]
#[require(Transform)]
pub struct CameraArm {
	pub direction: Dir3,
	pub distance: Units,
	pub sensitivity: Units,
}

impl Default for CameraArm {
	fn default() -> Self {
		Self {
			direction: Dir3::Z,
			distance: Units::from_u8(15),
			sensitivity: Units::from_u8(1),
		}
	}
}

impl MoveArm for CameraArm {
	fn move_arm(&mut self, angle: Vec2Radians) {
		let mut arm = Transform::default().looking_to(self.direction, Vec3::Y);
		let angle = angle * *self.sensitivity;

		arm.rotate_y(-angle.x);
		arm.rotate_local_x(angle.y);

		self.direction = arm.forward();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::f32::consts::PI;
	use testing::assert_eq_approx;

	const QUARTER_CIRCLE: f32 = PI / 2.;
	const HALF_CIRCLE: f32 = PI;

	#[test]
	fn rotate_around_y() {
		let mut arm = CameraArm {
			distance: Units::from(3.),
			sensitivity: Units::from(1.),
			direction: Dir3::X,
		};

		arm.move_arm(Vec2Radians::new(QUARTER_CIRCLE, 0.));

		assert_eq_approx!(Dir3::Z, arm.direction, 0.00001);
	}

	#[test]
	fn rotate_around_x() {
		let mut arm = CameraArm {
			distance: Units::from(3.),
			sensitivity: Units::from(1.),
			direction: Dir3::X,
		};

		arm.move_arm(Vec2Radians::new(0., QUARTER_CIRCLE));

		assert_eq_approx!(Dir3::Y, arm.direction, 0.00001);
	}

	#[test]
	fn rotate_with_sensitivity() {
		let mut arm = CameraArm {
			distance: Units::from(1.),
			sensitivity: Units::from(0.5),
			direction: Dir3::X,
		};

		arm.move_arm(Vec2Radians::new(HALF_CIRCLE, HALF_CIRCLE));

		assert_eq_approx!(Dir3::Y, arm.direction, 0.00001);
	}
}
