use super::{Movement, Units};
use crate::components::SimpleMovement;
use bevy::prelude::*;

impl Movement for SimpleMovement {
	fn update(&mut self, agent: &mut Transform, distance: Units) {
		let Some(target) = self.target else {
			return;
		};
		let direction = target - agent.translation;

		if distance < direction.length() {
			agent.translation += direction.normalize() * distance;
			return;
		}

		agent.translation = target;
		self.target = None;
	}
}

#[cfg(test)]
mod tests {
	use bevy::prelude::{Transform, Vec3};

	use crate::{components::SimpleMovement, traits::movement::Movement};

	#[test]
	fn move_to_target() {
		let mut movement = SimpleMovement {
			target: Some(Vec3::X),
		};
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 1.);

		assert_eq!(Vec3::X, agent.translation);
	}

	#[test]
	fn do_not_move_fully_if_distance_too_small() {
		let mut movement = SimpleMovement {
			target: Some(Vec3::new(2., 0., 0.)),
		};
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 0.5);

		assert_eq!(Vec3::X * 0.5, agent.translation);
	}

	#[test]
	fn do_not_overshoot() {
		let mut movement = SimpleMovement {
			target: Some(Vec3::X),
		};
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 100.);

		assert_eq!(Vec3::X, agent.translation);
	}

	#[test]
	fn do_not_move_when_no_target() {
		let mut movement = SimpleMovement { target: None };
		let mut agent = Transform::from_translation(Vec3::Y);

		movement.update(&mut agent, 1.);

		assert_eq!(Vec3::Y, agent.translation);
	}

	#[test]
	fn set_target_none_when_target_reached() {
		let mut movement = SimpleMovement {
			target: Some(Vec3::ONE),
		};
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 100.);

		assert!(movement.target.is_none());
	}
}
