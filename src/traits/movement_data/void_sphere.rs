use super::MovementData;
use crate::components::{Animate, UnitsPerSecond, VoidSphere};

const VOID_SPHERE_MOVE_SPEED: f32 = 1.;

impl MovementData<()> for VoidSphere {
	fn get_movement_data(&self) -> (UnitsPerSecond, Animate<()>) {
		(UnitsPerSecond::new(VOID_SPHERE_MOVE_SPEED), Animate::None)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_movement_data() {
		let void_sphere = VoidSphere;

		assert_eq!(
			(UnitsPerSecond::new(VOID_SPHERE_MOVE_SPEED), Animate::None),
			void_sphere.get_movement_data(),
		)
	}
}