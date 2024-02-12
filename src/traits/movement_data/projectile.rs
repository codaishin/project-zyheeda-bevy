use super::MovementData;
use crate::components::{Animate, Projectile};
use common::tools::UnitsPerSecond;

const PROJECTILE_MOVE_SPEED: f32 = 15.;

impl<T> MovementData<()> for Projectile<T> {
	fn get_movement_data(&self) -> (UnitsPerSecond, Animate<()>) {
		(UnitsPerSecond::new(PROJECTILE_MOVE_SPEED), Animate::None)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;

	#[test]
	fn move_fast() {
		let projectile = Projectile::<()>::new(default(), default());

		assert_eq!(
			(UnitsPerSecond::new(PROJECTILE_MOVE_SPEED), Animate::None),
			projectile.get_movement_data()
		);
	}
}
