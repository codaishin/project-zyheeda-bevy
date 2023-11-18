use super::MovementData;
use crate::{
	behaviors::MovementMode,
	components::{Projectile, UnitsPerSecond},
};

const PROJECTILE_MOVE_SPEED: f32 = 10.;

impl MovementData for Projectile {
	fn get_movement_data(&self) -> (UnitsPerSecond, MovementMode) {
		(
			UnitsPerSecond::new(PROJECTILE_MOVE_SPEED),
			MovementMode::Fast,
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::math::Ray;

	#[test]
	fn move_fast() {
		let projectile = Projectile {
			ray: Ray::default(),
		};

		assert_eq!(
			(
				UnitsPerSecond::new(PROJECTILE_MOVE_SPEED),
				MovementMode::Fast
			),
			projectile.get_movement_data()
		);
	}
}
