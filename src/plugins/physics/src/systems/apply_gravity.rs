use crate::components::{character_gravity::CharacterGravity, motion_controller::MotionController};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::time::Duration;

pub(crate) const GRAVITY: f32 = 1.;

impl MotionController {
	pub(crate) fn apply_gravity(
		delta: In<Duration>,
		characters: Query<(&mut CharacterGravity, &Self)>,
		mut controllers: Query<(
			&mut KinematicCharacterController,
			&KinematicCharacterControllerOutput,
		)>,
	) {
		let delta_secs = delta.as_secs_f32();

		for (mut gravity, controller) in characters {
			let Ok((mut controller, state)) = controllers.get_mut(controller.id()) else {
				continue;
			};

			if state.grounded {
				*gravity = CharacterGravity(0.);
				continue;
			}

			let new_gravity = gravity.0 + GRAVITY * delta_secs;

			let translation = match controller.translation {
				Some(translation) => translation.with_y(translation.y - new_gravity),
				None => Vec3::new(0., -new_gravity, 0.),
			};

			controller.translation = Some(translation);
			*gravity = CharacterGravity(new_gravity);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::motion_controller::MotionControllerOf;
	use bevy_rapier3d::prelude::KinematicCharacterController;
	use test_case::test_case;
	use testing::SingleThreadedApp;

	fn setup(delta: Duration) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(move || delta).pipe(MotionController::apply_gravity),
		);

		app
	}

	#[test_case(Duration::from_secs(1); "1 sec delta")]
	#[test_case(Duration::from_millis(100); "100 millis delta")]
	fn apply_gravity(delta: Duration) {
		let mut app = setup(delta);
		let agent = app.world_mut().spawn(CharacterGravity::default()).id();
		let entity = app
			.world_mut()
			.spawn((
				MotionControllerOf(agent),
				KinematicCharacterControllerOutput::default(),
				KinematicCharacterController::default(),
			))
			.id();

		app.update();

		assert_eq!(
			Some(Vec3::new(0., -GRAVITY * delta.as_secs_f32(), 0.,)),
			app.world()
				.entity(entity)
				.get::<KinematicCharacterController>()
				.and_then(|c| c.translation),
		);
	}

	#[test_case(Duration::from_secs(1); "1 sec delta")]
	#[test_case(Duration::from_millis(100); "100 millis delta")]
	fn accumulate_gravity(delta: Duration) {
		let mut app = setup(delta);
		let agent = app.world_mut().spawn(CharacterGravity(10.)).id();
		let entity = app
			.world_mut()
			.spawn((
				MotionControllerOf(agent),
				KinematicCharacterControllerOutput::default(),
				KinematicCharacterController::default(),
			))
			.id();

		app.update();

		assert_eq!(
			(
				Some(Vec3::new(0., -(10. + GRAVITY * delta.as_secs_f32()), 0.,)),
				Some(&CharacterGravity(10. + GRAVITY * delta.as_secs_f32())),
			),
			(
				app.world()
					.entity(entity)
					.get::<KinematicCharacterController>()
					.and_then(|c| c.translation),
				app.world().entity(agent).get::<CharacterGravity>(),
			)
		);
	}

	#[test_case(Duration::from_secs(1); "1 sec delta")]
	#[test_case(Duration::from_millis(100); "100 millis delta")]
	fn add_gravity(delta: Duration) {
		let mut app = setup(delta);
		let agent = app.world_mut().spawn(CharacterGravity::default()).id();
		let entity = app
			.world_mut()
			.spawn((
				MotionControllerOf(agent),
				KinematicCharacterControllerOutput::default(),
				KinematicCharacterController {
					translation: Some(Vec3::new(1., 2., 3.)),
					..default()
				},
			))
			.id();

		app.update();

		assert_eq!(
			Some(Vec3::new(1., 2. - GRAVITY * delta.as_secs_f32(), 3.,)),
			app.world()
				.entity(entity)
				.get::<KinematicCharacterController>()
				.and_then(|c| c.translation),
		);
	}

	#[test_case(Duration::from_secs(1); "1 sec delta")]
	#[test_case(Duration::from_millis(100); "100 millis delta")]
	fn add_accumulated_gravity(delta: Duration) {
		let mut app = setup(delta);
		let agent = app.world_mut().spawn(CharacterGravity(10.)).id();
		let entity = app
			.world_mut()
			.spawn((
				MotionControllerOf(agent),
				KinematicCharacterControllerOutput::default(),
				KinematicCharacterController {
					translation: Some(Vec3::new(1., 2., 3.)),
					..default()
				},
			))
			.id();

		app.update();

		assert_eq!(
			(
				Some(Vec3::new(
					1.,
					2. - (10. + GRAVITY * delta.as_secs_f32()),
					3.,
				)),
				Some(&CharacterGravity(10. + GRAVITY * delta.as_secs_f32())),
			),
			(
				app.world()
					.entity(entity)
					.get::<KinematicCharacterController>()
					.and_then(|c| c.translation),
				app.world().entity(agent).get::<CharacterGravity>(),
			)
		);
	}

	#[test]
	fn no_gravity_when_grounded() {
		let mut app = setup(Duration::from_secs(1));
		let agent = app.world_mut().spawn(CharacterGravity(11.)).id();
		let entity = app
			.world_mut()
			.spawn((
				MotionControllerOf(agent),
				KinematicCharacterControllerOutput {
					grounded: true,
					..default()
				},
				KinematicCharacterController::default(),
			))
			.id();

		app.update();

		assert_eq!(
			(None, Some(&CharacterGravity(0.))),
			(
				app.world()
					.entity(entity)
					.get::<KinematicCharacterController>()
					.and_then(|c| c.translation),
				app.world().entity(agent).get::<CharacterGravity>()
			)
		);
	}
}
