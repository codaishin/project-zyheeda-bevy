use crate::components::character_gravity::CharacterGravity;
use bevy::prelude::*;
use bevy_rapier3d::prelude::KinematicCharacterController;
use std::time::Duration;

impl CharacterGravity {
	const GRAVITY_STRENGTHS: f32 = 10.;

	pub(crate) fn apply(
		delta: In<Duration>,
		characters: Query<&mut KinematicCharacterController, With<Self>>,
	) {
		let gravity_strength = Self::GRAVITY_STRENGTHS * delta.as_secs_f32();

		for mut character in characters {
			let translation = match character.translation {
				Some(translation) => translation.with_y(translation.y - gravity_strength),
				None => Vec3::new(0., -gravity_strength, 0.),
			};

			character.translation = Some(translation);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy_rapier3d::prelude::KinematicCharacterController;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup(delta: Duration) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, (move || delta).pipe(CharacterGravity::apply));

		app
	}

	#[test_case(Duration::from_secs(1); "1 sec delta")]
	#[test_case(Duration::from_millis(100); "100 millis delta")]
	fn apply_gravity(delta: Duration) {
		let mut app = setup(delta);
		let entity = app
			.world_mut()
			.spawn((CharacterGravity, KinematicCharacterController::default()))
			.id();

		app.update();

		assert_eq!(
			Some(Vec3::NEG_Y * CharacterGravity::GRAVITY_STRENGTHS * delta.as_secs_f32()),
			app.world()
				.entity(entity)
				.get::<KinematicCharacterController>()
				.and_then(|c| c.translation),
		);
	}

	#[test_case(Duration::from_secs(1); "1 sec delta")]
	#[test_case(Duration::from_millis(100); "100 millis delta")]
	fn add_gravity(delta: Duration) {
		let mut app = setup(delta);
		let entity = app
			.world_mut()
			.spawn((
				CharacterGravity,
				KinematicCharacterController {
					translation: Some(Vec3::new(1., 2., 3.)),
					..default()
				},
			))
			.id();

		app.update();

		assert_eq!(
			Some(
				Vec3::new(1., 2., 3.)
					+ (Vec3::NEG_Y * CharacterGravity::GRAVITY_STRENGTHS * delta.as_secs_f32())
			),
			app.world()
				.entity(entity)
				.get::<KinematicCharacterController>()
				.and_then(|c| c.translation),
		);
	}

	#[test]
	fn do_nothing_when_character_gravity_missing() {
		let mut app = setup(Duration::from_secs(1));
		let entity = app
			.world_mut()
			.spawn(KinematicCharacterController::default())
			.id();

		app.update();

		assert_eq!(
			None,
			app.world()
				.entity(entity)
				.get::<KinematicCharacterController>()
				.and_then(|c| c.translation),
		);
	}
}
