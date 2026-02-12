use crate::components::character_motion::ApplyCharacterMotion;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::traits::handles_physics::CharacterMotion;
use std::time::Duration;

impl ApplyCharacterMotion {
	pub(crate) fn execute(
		delta: In<Duration>,
		characters: Query<(&mut KinematicCharacterController, &Transform, &Self)>,
	) {
		for (mut character, transform, motion) in characters {
			let translation = match motion {
				ApplyCharacterMotion::Ongoing(CharacterMotion::Direction { speed, direction }) => {
					*direction * **speed * delta.as_secs_f32()
				}
				ApplyCharacterMotion::Ongoing(CharacterMotion::ToTarget { speed, target }) => {
					(target - transform.translation)
						.try_normalize()
						.unwrap_or_default()
						* **speed * delta.as_secs_f32()
				}
				_ => continue,
			};

			character.translation = Some(translation);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::tools::{UnitsPerSecond, speed::Speed};
	use testing::SingleThreadedApp;

	fn setup(delta: Duration) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, (move || delta).pipe(ApplyCharacterMotion::execute));

		app
	}

	mod target_motion {
		use super::*;

		#[test]
		fn set_translation() {
			let delta = Duration::from_millis(100);
			let mut app = setup(delta);
			let entity = app
				.world_mut()
				.spawn((
					Transform::from_xyz(1., 2., 3.),
					KinematicCharacterController::default(),
					ApplyCharacterMotion::Ongoing(CharacterMotion::ToTarget {
						speed: Speed(UnitsPerSecond::from(1.)),
						target: Vec3::new(3., -1., 11.),
					}),
				))
				.id();

			app.update();

			assert_eq!(
				Some(Vec3::new(2., -3., 8.).normalize() * 0.1),
				app.world()
					.entity(entity)
					.get::<KinematicCharacterController>()
					.and_then(|c| c.translation),
			);
		}

		#[test]
		fn set_translation_with_speed() {
			let delta = Duration::from_millis(100);
			let mut app = setup(delta);
			let entity = app
				.world_mut()
				.spawn((
					Transform::from_xyz(1., 2., 3.),
					KinematicCharacterController::default(),
					ApplyCharacterMotion::Ongoing(CharacterMotion::ToTarget {
						speed: Speed(UnitsPerSecond::from(2.)),
						target: Vec3::new(3., -1., 11.),
					}),
				))
				.id();

			app.update();

			assert_eq!(
				Some(Vec3::new(2., -3., 8.).normalize() * 0.2),
				app.world()
					.entity(entity)
					.get::<KinematicCharacterController>()
					.and_then(|c| c.translation),
			);
		}
	}

	mod direction_motion {
		use super::*;

		#[test]
		fn set_translation() {
			let delta = Duration::from_millis(100);
			let mut app = setup(delta);
			let entity = app
				.world_mut()
				.spawn((
					Transform::default(),
					KinematicCharacterController::default(),
					ApplyCharacterMotion::Ongoing(CharacterMotion::Direction {
						speed: Speed(UnitsPerSecond::from(1.)),
						direction: Dir3::NEG_Y,
					}),
				))
				.id();

			app.update();

			assert_eq!(
				Some(Vec3::NEG_Y * 0.1),
				app.world()
					.entity(entity)
					.get::<KinematicCharacterController>()
					.and_then(|c| c.translation),
			);
		}

		#[test]
		fn set_translation_with_speed() {
			let delta = Duration::from_millis(100);
			let mut app = setup(delta);
			let entity = app
				.world_mut()
				.spawn((
					Transform::default(),
					KinematicCharacterController::default(),
					ApplyCharacterMotion::Ongoing(CharacterMotion::Direction {
						speed: Speed(UnitsPerSecond::from(2.)),
						direction: Dir3::NEG_Y,
					}),
				))
				.id();

			app.update();

			assert_eq!(
				Some(Vec3::NEG_Y * 0.2),
				app.world()
					.entity(entity)
					.get::<KinematicCharacterController>()
					.and_then(|c| c.translation),
			);
		}
	}
}
