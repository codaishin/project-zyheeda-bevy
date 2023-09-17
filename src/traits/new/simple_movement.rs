use bevy::prelude::Vec3;

use crate::behaviors::SimpleMovement;

use super::New1;

impl New1<Vec3> for SimpleMovement {
	fn new(target: Vec3) -> Self {
		Self {
			target: Some(target),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new_simple_move() {
		let movement = SimpleMovement::new(Vec3::new(3., 2., 1.));

		assert_eq!(Some(Vec3::new(3., 2., 1.)), movement.target);
	}
}