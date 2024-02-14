use super::{MovementData, ProjectileBehavior};
use bevy::{self, math::Vec3};
use common::{
	components::{Animate, Projectile},
	tools::UnitsPerSecond,
};

impl<T> ProjectileBehavior for Projectile<T> {
	fn direction(&self) -> Vec3 {
		self.direction
	}

	fn range(&self) -> f32 {
		self.range
	}
}

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
