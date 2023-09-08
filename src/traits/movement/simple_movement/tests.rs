use bevy::prelude::{Transform, Vec3};

use crate::{
	components::{SimpleMovement, UnitsPerSecond},
	traits::movement::Movement,
};

#[test]
fn move_to_target() {
	let movement = SimpleMovement {
		speed: UnitsPerSecond::new(1.),
	};
	let mut agent = Transform::from_translation(Vec3::ZERO);

	movement.move_towards(&mut agent, Vec3::X, 1.);

	assert_eq!(Vec3::X, agent.translation);
}

#[test]
fn move_to_target_with_appropriate_speed() {
	let movement = SimpleMovement {
		speed: UnitsPerSecond::new(0.5),
	};
	let mut agent = Transform::from_translation(Vec3::ZERO);

	movement.move_towards(&mut agent, Vec3::new(2., 0., 0.), 1.);

	assert_eq!(Vec3::X * 0.5, agent.translation);
}

#[test]
fn move_to_target_with_speed_scaled_on_delta() {
	let movement = SimpleMovement {
		speed: UnitsPerSecond::new(0.5),
	};
	let mut agent = Transform::from_translation(Vec3::ZERO);

	movement.move_towards(&mut agent, Vec3::new(2., 0., 0.), 0.5);

	assert_eq!(Vec3::X * 0.5 * 0.5, agent.translation);
}

#[test]
fn do_not_move_directly_if_speed_with_delta_too_small() {
	let movement = SimpleMovement {
		speed: UnitsPerSecond::new(2.),
	};
	let mut agent = Transform::from_translation(Vec3::ZERO);

	movement.move_towards(&mut agent, Vec3::new(2., 0., 0.), 0.5);

	assert_eq!(Vec3::X, agent.translation);
}

#[test]
fn do_not_overshoot() {
	let movement = SimpleMovement {
		speed: UnitsPerSecond::new(2.),
	};
	let mut agent = Transform::from_translation(Vec3::ZERO);

	movement.move_towards(&mut agent, Vec3::X, 1.);

	assert_eq!(Vec3::X, agent.translation);
}
