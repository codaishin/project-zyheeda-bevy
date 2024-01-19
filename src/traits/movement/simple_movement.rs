use super::{IsDone, Movement, Units};
use crate::components::SimpleMovement;
use bevy::prelude::*;

impl Movement for SimpleMovement {
	fn update(&mut self, agent: &mut Transform, distance: Units) -> IsDone {
		let target = self.target;
		let direction = target - agent.translation;

		if distance > direction.length() {
			agent.translation = target;
			return true;
		}

		agent.look_at(Vec3::new(target.x, agent.translation.y, target.z), Vec3::Y);
		agent.translation += direction.normalize() * distance;
		false
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_tools::utils::assert_eq_approx;
	use bevy::prelude::{Transform, Vec3};

	#[test]
	fn move_to_target() {
		let mut movement = SimpleMovement::new(Vec3::X);
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 1.);

		assert_eq!(Vec3::X, agent.translation);
	}

	#[test]
	fn do_not_move_fully_if_distance_too_small() {
		let mut movement = SimpleMovement::new(Vec3::new(2., 0., 0.));
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 0.5);

		assert_eq!(Vec3::X * 0.5, agent.translation);
	}

	#[test]
	fn do_not_overshoot() {
		let mut movement = SimpleMovement::new(Vec3::X);
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 100.);

		assert_eq!(Vec3::X, agent.translation);
	}

	#[test]
	fn done_when_target_reached() {
		let mut movement = SimpleMovement::new(Vec3::ONE);
		let mut agent = Transform::from_translation(Vec3::ZERO);

		let is_done = movement.update(&mut agent, 100.);

		assert!(is_done);
	}

	#[test]
	fn not_done_when_target_reached() {
		let mut movement = SimpleMovement::new(Vec3::ONE);
		let mut agent = Transform::from_translation(Vec3::ZERO);

		let is_done = movement.update(&mut agent, 0.1);

		assert!(!is_done);
	}

	#[test]
	fn set_forward() {
		let mut movement = SimpleMovement::new(Vec3::X);
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 0.1);

		assert_eq_approx!(Vec3::X, agent.forward(), 0.00001);
	}

	#[test]
	fn set_forward_ignoring_height_difference() {
		let mut movement = SimpleMovement::new(Vec3::X);
		let mut agent = Transform::from_translation(Vec3::new(0., -1., 0.));

		movement.update(&mut agent, 0.1);

		assert_eq_approx!(Vec3::X, agent.forward(), 0.00001);
	}

	#[test]
	fn no_rotation_change_when_on_target() {
		let mut movement = SimpleMovement::new(Vec3::ONE);
		let mut agent = Transform::from_translation(Vec3::ONE);

		agent.look_at(Vec3::new(2., 1., 2.), Vec3::Y);

		movement.update(&mut agent, 0.1);

		assert_eq_approx!(Vec3::new(2., 0., 2.).normalize(), agent.forward(), 0.00001);
	}
}
