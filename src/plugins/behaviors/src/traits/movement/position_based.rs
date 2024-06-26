use crate::{
	components::{Movement, PositionBased},
	traits::{IsDone, MovementPositionBased, Units},
};
use bevy::prelude::*;
use std::ops::Deref;

impl MovementPositionBased for Movement<PositionBased> {
	fn update(&mut self, agent: &mut Transform, distance: Units) -> IsDone {
		let target = self.target;
		let direction = target - agent.translation;
		let distance = *distance.deref();

		if distance > direction.length() {
			agent.translation = target;
			return IsDone::from(true);
		}

		agent.translation += direction.normalize() * distance;
		IsDone::from(false)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::prelude::{Transform, Vec3};
	use common::{
		test_tools::utils::assert_eq_approx,
		traits::clamp_zero_positive::ClampZeroPositive,
	};

	#[test]
	fn move_to_target() {
		let mut movement = Movement::<PositionBased>::to(Vec3::X);
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, Units::new(1.));

		assert_eq!(Vec3::X, agent.translation);
	}

	#[test]
	fn do_not_move_fully_if_distance_too_small() {
		let mut movement = Movement::<PositionBased>::to(Vec3::new(2., 0., 0.));
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, Units::new(0.5));

		assert_eq!(Vec3::X * 0.5, agent.translation);
	}

	#[test]
	fn do_not_overshoot() {
		let mut movement = Movement::<PositionBased>::to(Vec3::X);
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, Units::new(100.));

		assert_eq!(Vec3::X, agent.translation);
	}

	#[test]
	fn done_when_target_reached() {
		let mut movement = Movement::<PositionBased>::to(Vec3::ONE);
		let mut agent = Transform::from_translation(Vec3::ZERO);

		assert!(movement.update(&mut agent, Units::new(100.)).is_done());
	}

	#[test]
	fn not_done_when_target_reached() {
		let mut movement = Movement::<PositionBased>::to(Vec3::ONE);
		let mut agent = Transform::from_translation(Vec3::ZERO);

		assert!(!movement.update(&mut agent, Units::new(0.1)).is_done());
	}

	#[test]
	fn no_rotation_change_when_on_target() {
		let mut movement = Movement::<PositionBased>::to(Vec3::ONE);
		let mut agent = Transform::from_translation(Vec3::ONE);

		agent.look_at(Vec3::new(2., 1., 2.), Vec3::Y);

		movement.update(&mut agent, Units::new(0.1));

		assert_eq_approx!(Vec3::new(2., 0., 2.).normalize(), agent.forward(), 0.00001);
	}
}
