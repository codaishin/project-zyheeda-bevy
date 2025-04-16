use bevy::{math::InvalidDirectionError, prelude::*};
use common::{tools::keys::movement::MovementKey, traits::handles_player::KeyDirection};

#[derive(Component, Debug, PartialEq, Default)]
pub struct PlayerCamera;

impl KeyDirection<MovementKey> for PlayerCamera {
	fn key_direction(
		cam_transform: &GlobalTransform,
		movement_key: &MovementKey,
	) -> Result<Dir3, InvalidDirectionError> {
		let direction = match movement_key {
			MovementKey::Forward => *cam_transform.forward() + *cam_transform.up(),
			MovementKey::Backward => *cam_transform.back() + *cam_transform.down(),
			MovementKey::Left => *cam_transform.left(),
			MovementKey::Right => *cam_transform.right(),
		};

		Dir3::try_from(direction.with_y(0.))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::assert_eq_approx;

	#[test]
	fn get_directions_default() {
		let transform = GlobalTransform::default();

		assert_eq!(
			[Ok(Dir3::NEG_Z), Ok(Dir3::Z), Ok(Dir3::NEG_X), Ok(Dir3::X)],
			[
				PlayerCamera::key_direction(&transform, &MovementKey::Forward),
				PlayerCamera::key_direction(&transform, &MovementKey::Backward),
				PlayerCamera::key_direction(&transform, &MovementKey::Left),
				PlayerCamera::key_direction(&transform, &MovementKey::Right),
			]
		);
	}

	#[test]
	fn get_directions_when_looking_right() {
		let transform = GlobalTransform::from(Transform::default().looking_to(Dir3::X, Vec3::Y));

		assert_eq_approx!(
			[Ok(Dir3::X), Ok(Dir3::NEG_X), Ok(Dir3::NEG_Z), Ok(Dir3::Z)],
			[
				PlayerCamera::key_direction(&transform, &MovementKey::Forward),
				PlayerCamera::key_direction(&transform, &MovementKey::Backward),
				PlayerCamera::key_direction(&transform, &MovementKey::Left),
				PlayerCamera::key_direction(&transform, &MovementKey::Right),
			],
			f32::EPSILON
		);
	}

	#[test]
	fn get_directions_horizontal_when_looking_forward_down() {
		let transform = GlobalTransform::from(
			Transform::default().looking_at(Vec3::new(0., -1., -1.), Vec3::new(0., 1., -1.)),
		);

		assert_eq_approx!(
			[Ok(Dir3::NEG_Z), Ok(Dir3::Z), Ok(Dir3::NEG_X), Ok(Dir3::X)],
			[
				PlayerCamera::key_direction(&transform, &MovementKey::Forward),
				PlayerCamera::key_direction(&transform, &MovementKey::Backward),
				PlayerCamera::key_direction(&transform, &MovementKey::Left),
				PlayerCamera::key_direction(&transform, &MovementKey::Right),
			],
			f32::EPSILON
		);
	}

	#[test]
	fn get_directions_horizontal_when_looking_down() {
		let transform =
			GlobalTransform::from(Transform::default().looking_to(Dir3::NEG_Y, Vec3::NEG_Z));

		assert_eq_approx!(
			[Ok(Dir3::NEG_Z), Ok(Dir3::Z), Ok(Dir3::NEG_X), Ok(Dir3::X)],
			[
				PlayerCamera::key_direction(&transform, &MovementKey::Forward),
				PlayerCamera::key_direction(&transform, &MovementKey::Backward),
				PlayerCamera::key_direction(&transform, &MovementKey::Left),
				PlayerCamera::key_direction(&transform, &MovementKey::Right),
			],
			f32::EPSILON
		);
	}

	#[test]
	fn get_directions_horizontal_when_looking_down_and_rotated_right() {
		let transform =
			GlobalTransform::from(Transform::default().looking_to(Dir3::NEG_Y, Vec3::X));

		assert_eq_approx!(
			[Ok(Dir3::X), Ok(Dir3::NEG_X), Ok(Dir3::NEG_Z), Ok(Dir3::Z)],
			[
				PlayerCamera::key_direction(&transform, &MovementKey::Forward),
				PlayerCamera::key_direction(&transform, &MovementKey::Backward),
				PlayerCamera::key_direction(&transform, &MovementKey::Left),
				PlayerCamera::key_direction(&transform, &MovementKey::Right),
			],
			f32::EPSILON
		);
	}
}
