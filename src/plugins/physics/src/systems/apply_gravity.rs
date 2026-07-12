use crate::components::{
	character_gravity::CharacterGravity,
	motion_controller::{MotionController, MotionControllerOf},
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use std::time::Duration;

pub(crate) const GROUNDED_GRAVITY: f32 = 0.1;
pub(crate) const FALL_GRAVITY: f32 = 1.;

impl MotionControllerOf {
	pub(crate) fn apply_gravity(
		delta: In<Duration>,
		characters: Query<(&mut CharacterGravity, &MotionController)>,
		mut controllers: Query<(
			&mut KinematicCharacterController,
			&KinematicCharacterControllerOutput,
		)>,
	) {
		let delta_secs = delta.as_secs_f32();

		for (mut gravity, controller) in characters {
			let Ok((mut ctrl, ctrl_state)) = controllers.get_mut(controller.get()) else {
				continue;
			};

			let new_gravity = match ctrl_state.grounded {
				true => GROUNDED_GRAVITY * delta_secs,
				false => gravity.0 + FALL_GRAVITY * delta_secs,
			};

			let translation = match ctrl.translation {
				Some(translation) => translation.with_y(translation.y - new_gravity),
				None => Vec3::new(0., -new_gravity, 0.),
			};

			ctrl.translation = Some(translation);
			*gravity = CharacterGravity(new_gravity);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy_rapier3d::prelude::KinematicCharacterController;
	use testing::SingleThreadedApp;

	fn setup(delta: Duration) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(move || delta).pipe(MotionControllerOf::apply_gravity),
		);

		app
	}

	mod fall_gravity {
		use super::*;
		use crate::components::motion_controller::MotionControllerOf;
		use test_case::test_case;

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
				Some(Vec3::new(0., -FALL_GRAVITY * delta.as_secs_f32(), 0.,)),
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
					Some(Vec3::new(
						0.,
						-(10. + FALL_GRAVITY * delta.as_secs_f32()),
						0.,
					)),
					Some(&CharacterGravity(10. + FALL_GRAVITY * delta.as_secs_f32())),
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
				Some(Vec3::new(1., 2. - FALL_GRAVITY * delta.as_secs_f32(), 3.,)),
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
						2. - (10. + FALL_GRAVITY * delta.as_secs_f32()),
						3.,
					)),
					Some(&CharacterGravity(10. + FALL_GRAVITY * delta.as_secs_f32())),
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
	}

	mod grounded_gravity {
		use crate::components::motion_controller::MotionControllerOf;

		use super::*;
		use test_case::test_case;

		#[test_case(Duration::from_secs(1); "1 sec delta")]
		#[test_case(Duration::from_millis(100); "100 millis delta")]
		fn apply_gravity(delta: Duration) {
			let mut app = setup(delta);
			let agent = app.world_mut().spawn(CharacterGravity::default()).id();
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
				Some(Vec3::new(0., -GROUNDED_GRAVITY * delta.as_secs_f32(), 0.)),
				app.world()
					.entity(entity)
					.get::<KinematicCharacterController>()
					.and_then(|c| c.translation),
			);
		}

		#[test_case(Duration::from_secs(1); "1 sec delta")]
		#[test_case(Duration::from_millis(100); "100 millis delta")]
		fn reset_accumulated_gravity(delta: Duration) {
			let mut app = setup(delta);
			let agent = app.world_mut().spawn(CharacterGravity(10.)).id();
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
				(
					Some(Vec3::new(0., -GROUNDED_GRAVITY * delta.as_secs_f32(), 0.)),
					Some(&CharacterGravity(GROUNDED_GRAVITY * delta.as_secs_f32())),
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
				Some(Vec3::new(
					1.,
					2. - GROUNDED_GRAVITY * delta.as_secs_f32(),
					3.,
				)),
				app.world()
					.entity(entity)
					.get::<KinematicCharacterController>()
					.and_then(|c| c.translation),
			);
		}

		#[test_case(Duration::from_secs(1); "1 sec delta")]
		#[test_case(Duration::from_millis(100); "100 millis delta")]
		fn reset_added_accumulated_gravity(delta: Duration) {
			let mut app = setup(delta);
			let agent = app.world_mut().spawn(CharacterGravity(10.)).id();
			let entity = app
				.world_mut()
				.spawn((
					MotionControllerOf(agent),
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
				(
					Some(Vec3::new(
						1.,
						2. - GROUNDED_GRAVITY * delta.as_secs_f32(),
						3.,
					)),
					Some(&CharacterGravity(GROUNDED_GRAVITY * delta.as_secs_f32())),
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
	}
}
