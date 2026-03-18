use crate::components::character_gravity::CharacterGravity;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::time::Duration;

impl CharacterGravity {
	const GROUNDED_GRAVITY: f32 = 0.01;
	const FALL_GRAVITY: f32 = 10.;

	pub(crate) fn apply(
		delta: In<Duration>,
		characters: Query<
			(
				&mut KinematicCharacterController,
				&KinematicCharacterControllerOutput,
			),
			With<Self>,
		>,
	) {
		let delta_secs = delta.as_secs_f32();

		for (mut character, state) in characters {
			let gravity = match state.grounded {
				true => Self::GROUNDED_GRAVITY,
				false => Self::FALL_GRAVITY,
			};

			let translation = match character.translation {
				Some(translation) => translation.with_y(translation.y - gravity * delta_secs),
				None => Vec3::new(0., -gravity * delta_secs, 0.),
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
	fn apply_fall_gravity(delta: Duration) {
		let mut app = setup(delta);
		let entity = app
			.world_mut()
			.spawn((
				CharacterGravity,
				KinematicCharacterControllerOutput::default(),
				KinematicCharacterController::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(Vec3::NEG_Y * CharacterGravity::FALL_GRAVITY * delta.as_secs_f32()),
			app.world()
				.entity(entity)
				.get::<KinematicCharacterController>()
				.and_then(|c| c.translation),
		);
	}

	#[test_case(Duration::from_secs(1); "1 sec delta")]
	#[test_case(Duration::from_millis(100); "100 millis delta")]
	fn apply_grounded_gravity(delta: Duration) {
		let mut app = setup(delta);
		let entity = app
			.world_mut()
			.spawn((
				CharacterGravity,
				KinematicCharacterControllerOutput {
					grounded: true,
					..default()
				},
				KinematicCharacterController::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(Vec3::NEG_Y * CharacterGravity::GROUNDED_GRAVITY * delta.as_secs_f32()),
			app.world()
				.entity(entity)
				.get::<KinematicCharacterController>()
				.and_then(|c| c.translation),
		);
	}

	#[test_case(Duration::from_secs(1); "1 sec delta")]
	#[test_case(Duration::from_millis(100); "100 millis delta")]
	fn add_fall_gravity(delta: Duration) {
		let mut app = setup(delta);
		let entity = app
			.world_mut()
			.spawn((
				CharacterGravity,
				KinematicCharacterControllerOutput::default(),
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
					+ (Vec3::NEG_Y * CharacterGravity::FALL_GRAVITY * delta.as_secs_f32())
			),
			app.world()
				.entity(entity)
				.get::<KinematicCharacterController>()
				.and_then(|c| c.translation),
		);
	}

	#[test_case(Duration::from_secs(1); "1 sec delta")]
	#[test_case(Duration::from_millis(100); "100 millis delta")]
	fn add_grounded_gravity(delta: Duration) {
		let mut app = setup(delta);
		let entity = app
			.world_mut()
			.spawn((
				CharacterGravity,
				KinematicCharacterControllerOutput {
					grounded: true,
					..default()
				},
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
					+ (Vec3::NEG_Y * CharacterGravity::GROUNDED_GRAVITY * delta.as_secs_f32())
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
			.spawn((
				KinematicCharacterControllerOutput::default(),
				KinematicCharacterController::default(),
			))
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
