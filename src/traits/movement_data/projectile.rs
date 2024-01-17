use super::MovementData;
use crate::components::{Animate, Projectile, UnitsPerSecond};

const PROJECTILE_MOVE_SPEED: f32 = 10.;

impl MovementData<()> for Projectile {
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
		let projectile = Projectile { ..default() };

		assert_eq!(
			(UnitsPerSecond::new(PROJECTILE_MOVE_SPEED), Animate::None),
			projectile.get_movement_data()
		);
	}
}
