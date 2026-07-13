use crate::components::{
	character_motion::{ApplyMotion, IsInMotion},
	immobilized::Immobilized,
	motion_controller::MotionController,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use common::traits::handles_physics::CharacterMotion;
use std::time::Duration;

impl MotionController {
	#[allow(clippy::type_complexity)]
	pub(crate) fn set_translation(
		delta: In<Duration>,
		agents: Query<
			(&ApplyMotion, &mut Transform, &Self),
			(Without<Immobilized>, With<IsInMotion>),
		>,
		mut controllers: Query<
			(&mut KinematicCharacterController, &Transform),
			Without<ApplyMotion>,
		>,
	) {
		for (ApplyMotion(motion), mut transform, ctrl) in agents {
			let Ok((mut ctrl, ctrl_transform)) = controllers.get_mut(ctrl.get()) else {
				continue;
			};

			let target_translation = match motion {
				CharacterMotion::Direction { speed, direction } => {
					*direction * **speed * delta.as_secs_f32()
				}
				CharacterMotion::ToTarget { speed, target } => {
					(target - ctrl_transform.translation)
						.try_normalize()
						.unwrap_or_default()
						* **speed * delta.as_secs_f32()
				}
				CharacterMotion::Done => continue,
			};

			transform.translation = ctrl_transform.translation;
			ctrl.translation = Some(target_translation);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::motion_controller::MotionControllerOf;
	use common::tools::{UnitsPerSecond, speed::Speed};
	use std::f32::consts::PI;
	use testing::SingleThreadedApp;

	fn setup(delta: Duration) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(move || delta).pipe(MotionController::set_translation),
		);

		app
	}

	mod target_motion {
		use super::*;

		#[test]
		fn set_agent_translation() {
			let delta = Duration::from_millis(100);
			let mut app = setup(delta);
			let agent = app
				.world_mut()
				.spawn(ApplyMotion::from(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::new(3., -1., 11.),
				}))
				.id();
			app.world_mut().spawn((
				MotionControllerOf(agent),
				Transform::from_xyz(1., 2., 3.),
				KinematicCharacterController::default(),
			));

			app.update();

			assert_eq!(
				Some(&Transform::from_xyz(1., 2., 3.)),
				app.world().entity(agent).get::<Transform>(),
			);
		}

		#[test]
		fn set_only_agent_translation() {
			let delta = Duration::from_millis(100);
			let mut app = setup(delta);
			let agent = app
				.world_mut()
				.spawn((
					Transform {
						translation: Vec3::ZERO,
						rotation: Quat::from_rotation_y(PI),
						scale: Vec3::splat(10.),
					},
					ApplyMotion::from(CharacterMotion::ToTarget {
						speed: Speed(UnitsPerSecond::from(1.)),
						target: Vec3::new(3., -1., 11.),
					}),
				))
				.id();
			app.world_mut().spawn((
				MotionControllerOf(agent),
				Transform::from_xyz(1., 2., 3.),
				KinematicCharacterController::default(),
			));

			app.update();

			assert_eq!(
				Some(&Transform {
					translation: Vec3::new(1., 2., 3.),
					rotation: Quat::from_rotation_y(PI),
					scale: Vec3::splat(10.),
				}),
				app.world().entity(agent).get::<Transform>(),
			);
		}

		#[test]
		fn set_target_translation() {
			let delta = Duration::from_millis(100);
			let mut app = setup(delta);
			let agent = app
				.world_mut()
				.spawn(ApplyMotion::from(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::new(3., -1., 11.),
				}))
				.id();
			let entity = app
				.world_mut()
				.spawn((
					MotionControllerOf(agent),
					Transform::from_xyz(1., 2., 3.),
					KinematicCharacterController::default(),
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
		fn set_target_translation_with_speed() {
			let delta = Duration::from_millis(100);
			let mut app = setup(delta);
			let agent = app
				.world_mut()
				.spawn(ApplyMotion::from(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(2.)),
					target: Vec3::new(3., -1., 11.),
				}))
				.id();
			let entity = app
				.world_mut()
				.spawn((
					MotionControllerOf(agent),
					Transform::from_xyz(1., 2., 3.),
					KinematicCharacterController::default(),
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
		fn set_agent_translation() {
			let delta = Duration::from_millis(100);
			let mut app = setup(delta);
			let agent = app
				.world_mut()
				.spawn(ApplyMotion::from(CharacterMotion::Direction {
					speed: Speed(UnitsPerSecond::from(1.)),
					direction: Dir3::NEG_Y,
				}))
				.id();
			app.world_mut().spawn((
				MotionControllerOf(agent),
				Transform::from_xyz(1., 2., 3.),
				KinematicCharacterController::default(),
			));

			app.update();

			assert_eq!(
				Some(&Transform::from_xyz(1., 2., 3.)),
				app.world().entity(agent).get::<Transform>(),
			);
		}

		#[test]
		fn set_only_agent_translation() {
			let delta = Duration::from_millis(100);
			let mut app = setup(delta);
			let agent = app
				.world_mut()
				.spawn((
					Transform {
						translation: Vec3::ZERO,
						rotation: Quat::from_rotation_y(PI),
						scale: Vec3::splat(10.),
					},
					ApplyMotion::from(CharacterMotion::Direction {
						speed: Speed(UnitsPerSecond::from(1.)),
						direction: Dir3::NEG_Y,
					}),
				))
				.id();
			app.world_mut().spawn((
				MotionControllerOf(agent),
				Transform::from_xyz(1., 2., 3.),
				KinematicCharacterController::default(),
			));

			app.update();

			assert_eq!(
				Some(&Transform {
					translation: Vec3::new(1., 2., 3.),
					rotation: Quat::from_rotation_y(PI),
					scale: Vec3::splat(10.),
				}),
				app.world().entity(agent).get::<Transform>(),
			);
		}

		#[test]
		fn set_target_translation() {
			let delta = Duration::from_millis(100);
			let mut app = setup(delta);
			let agent = app
				.world_mut()
				.spawn(ApplyMotion::from(CharacterMotion::Direction {
					speed: Speed(UnitsPerSecond::from(1.)),
					direction: Dir3::NEG_Y,
				}))
				.id();
			let entity = app
				.world_mut()
				.spawn((
					MotionControllerOf(agent),
					Transform::default(),
					KinematicCharacterController::default(),
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
		fn set_target_translation_with_speed() {
			let delta = Duration::from_millis(100);
			let mut app = setup(delta);
			let agent = app
				.world_mut()
				.spawn(ApplyMotion::from(CharacterMotion::Direction {
					speed: Speed(UnitsPerSecond::from(2.)),
					direction: Dir3::NEG_Y,
				}))
				.id();
			let entity = app
				.world_mut()
				.spawn((
					MotionControllerOf(agent),
					Transform::default(),
					KinematicCharacterController::default(),
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

	mod filters {
		use super::*;

		#[test]
		fn do_nothing_when_immobilized() {
			let mut app = setup(Duration::from_millis(100));
			let agent = app
				.world_mut()
				.spawn((
					Immobilized,
					ApplyMotion::from(CharacterMotion::ToTarget {
						speed: Speed(UnitsPerSecond::from(1.)),
						target: Vec3::new(3., -1., 11.),
					}),
				))
				.id();
			let entity = app
				.world_mut()
				.spawn((
					MotionControllerOf(agent),
					Transform::default(),
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

		#[test]
		fn do_nothing_not_in_motion() {
			let mut app = setup(Duration::from_millis(100));
			let agent = app
				.world_mut()
				.spawn(ApplyMotion::from(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::new(3., -1., 11.),
				}))
				.remove::<IsInMotion>()
				.id();
			let entity = app
				.world_mut()
				.spawn((
					MotionControllerOf(agent),
					Transform::default(),
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
}
