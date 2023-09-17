use super::{Movement, Seconds};
use crate::components::SimpleMovement;
use bevy::prelude::*;

impl Movement for SimpleMovement {
	fn update(&self, agent: &mut Transform, delta_time: Seconds) {
		let Some(target) = self.target else {
			return;
		};

		let direction = target - agent.translation;
		let distance = self.speed.unpack() * delta_time;

		match distance < direction.length() {
			true => agent.translation += direction.normalize() * distance,
			false => agent.translation = target,
		};
	}
}

#[cfg(test)]
mod tests {
	use bevy::prelude::{Transform, Vec3};

	use crate::{
		components::{SimpleMovement, UnitsPerSecond},
		traits::movement::Movement,
	};

	#[test]
	fn move_to_target() {
		let movement = SimpleMovement {
			target: Some(Vec3::X),
			speed: UnitsPerSecond::new(1.),
		};
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 1.);

		assert_eq!(Vec3::X, agent.translation);
	}

	#[test]
	fn move_to_target_with_appropriate_speed() {
		let movement = SimpleMovement {
			target: Some(Vec3::new(2., 0., 0.)),
			speed: UnitsPerSecond::new(0.5),
		};
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 1.);

		assert_eq!(Vec3::X * 0.5, agent.translation);
	}

	#[test]
	fn move_to_target_with_speed_scaled_on_delta() {
		let movement = SimpleMovement {
			target: Some(Vec3::new(2., 0., 0.)),
			speed: UnitsPerSecond::new(0.5),
		};
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 0.5);

		assert_eq!(Vec3::X * 0.5 * 0.5, agent.translation);
	}

	#[test]
	fn do_not_move_directly_if_speed_with_delta_too_small() {
		let movement = SimpleMovement {
			target: Some(Vec3::new(2., 0., 0.)),
			speed: UnitsPerSecond::new(2.),
		};
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 0.5);

		assert_eq!(Vec3::X, agent.translation);
	}

	#[test]
	fn do_not_overshoot() {
		let movement = SimpleMovement {
			target: Some(Vec3::X),
			speed: UnitsPerSecond::new(2.),
		};
		let mut agent = Transform::from_translation(Vec3::ZERO);

		movement.update(&mut agent, 1.);

		assert_eq!(Vec3::X, agent.translation);
	}

	#[test]
	fn do_not_move_when_no_target() {
		let movement = SimpleMovement {
			target: None,
			speed: UnitsPerSecond::new(2.),
		};
		let mut agent = Transform::from_translation(Vec3::Y);

		movement.update(&mut agent, 1.);

		assert_eq!(Vec3::Y, agent.translation);
	}
}
