use crate::components::{
	character_motion::{ApplyMotion, IsInMotion},
	motion_controller::{MotionController, OldTranslation},
};
use bevy::prelude::*;
use common::{
	traits::{accessors::get::TryApplyOn, handles_physics::CharacterMotion},
	zyheeda_commands::ZyheedaCommands,
};
use std::time::Duration;

impl MotionController {
	pub(crate) fn set_done(
		In(delta): In<Duration>,
		mut commands: ZyheedaCommands,
		controlled: Query<(Entity, &ApplyMotion, &Self), With<IsInMotion>>,
		mut transforms: Query<&mut Transform>,
		mut old_translation: Query<&mut OldTranslation>,
	) {
		for (entity, apply, ctrl) in controlled {
			let Ok(ctrl_transform) = transforms.get(ctrl.id()) else {
				continue;
			};
			let Ok(mut old_translation) = old_translation.get_mut(ctrl.id()) else {
				continue;
			};

			if !is_done(&apply.0, ctrl_transform, delta) {
				continue;
			}

			let ctrl_translation = ctrl_transform.translation;
			*old_translation = OldTranslation(ctrl_translation);
			if let Ok(mut transform) = transforms.get_mut(entity) {
				transform.translation = ctrl_translation;
			}

			commands.try_apply_on(&entity, |mut e| {
				e.try_insert(ApplyMotion(CharacterMotion::Done));
				e.try_remove::<IsInMotion>();
			});
		}
	}
}

const ALLOWED_HEIGHT_DIFFERENCE: f32 = 0.5;

fn is_done(motion: &CharacterMotion, transform: &Transform, delta: Duration) -> bool {
	let (speed, target) = match motion {
		CharacterMotion::Direction { .. } => return false,
		CharacterMotion::Done => return true,
		CharacterMotion::ToTarget { speed, target } => (speed, target),
	};

	if target == &transform.translation {
		return true;
	}

	if (target.y - transform.translation.y).abs() > ALLOWED_HEIGHT_DIFFERENCE {
		return false;
	}

	let distance_to_target = (target.xz() - transform.translation.xz()).length();
	let distance_traveled_per_frame = delta.as_secs_f32() * **speed;

	distance_to_target < distance_traveled_per_frame
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use std::f32::consts::PI;

	use super::*;
	use crate::components::{character_motion::IsInMotion, motion_controller::MotionControllerOf};
	use common::{
		tools::{UnitsPerSecond, speed::Speed},
		traits::handles_physics::CharacterMotion,
	};
	use testing::{IsChanged, SingleThreadedApp};

	fn setup(delta: Duration) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(
				(move || delta).pipe(MotionController::set_done),
				IsChanged::<ApplyMotion>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn remain_done_when_done() {
		let mut app = setup(Duration::default());
		let agent = app
			.world_mut()
			.spawn(ApplyMotion::from(CharacterMotion::Done))
			.id();
		app.world_mut()
			.spawn((MotionControllerOf(agent), Transform::default()));

		app.update();

		assert_eq!(
			(Some(&ApplyMotion(CharacterMotion::Done)), None),
			(
				app.world().entity(agent).get::<ApplyMotion>(),
				app.world().entity(agent).get::<IsInMotion>(),
			)
		);
	}

	#[test]
	fn set_done_when_translation_matches_target() {
		let mut app = setup(Duration::default());
		let agent = app
			.world_mut()
			.spawn((
				Transform {
					translation: Vec3::ZERO,
					scale: Vec3::splat(42.),
					rotation: Quat::from_rotation_y(PI),
				},
				ApplyMotion::from(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::new(1., 2., 3.),
				}),
			))
			.id();
		let ctrl = app
			.world_mut()
			.spawn((MotionControllerOf(agent), Transform::from_xyz(1., 2., 3.)))
			.id();

		app.update();

		assert_eq!(
			(
				Some(&ApplyMotion(CharacterMotion::Done)),
				None,
				Some(&Transform {
					translation: Vec3::new(1., 2., 3.),
					scale: Vec3::splat(42.),
					rotation: Quat::from_rotation_y(PI),
				}),
				Some(&OldTranslation(Vec3::new(1., 2., 3.))),
			),
			(
				app.world().entity(agent).get::<ApplyMotion>(),
				app.world().entity(agent).get::<IsInMotion>(),
				app.world().entity(agent).get::<Transform>(),
				app.world().entity(ctrl).get::<OldTranslation>(),
			)
		);
	}

	#[test]
	fn do_not_set_done_when_translation_does_not_match_target() {
		let mut app = setup(Duration::default());
		let agent = app
			.world_mut()
			.spawn((
				Transform::default(),
				ApplyMotion::from(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::new(10., 2., 3.),
				}),
			))
			.id();
		let ctrl = app
			.world_mut()
			.spawn((MotionControllerOf(agent), Transform::from_xyz(1., 2., 3.)))
			.id();

		app.update();

		assert_eq!(
			(
				Some(&ApplyMotion(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::new(10., 2., 3.),
				})),
				Some(&IsInMotion),
				Some(&Transform::default()),
				Some(&OldTranslation::default()),
			),
			(
				app.world().entity(agent).get::<ApplyMotion>(),
				app.world().entity(agent).get::<IsInMotion>(),
				app.world().entity(agent).get::<Transform>(),
				app.world().entity(ctrl).get::<OldTranslation>(),
			)
		);
	}

	#[test]
	fn set_done_when_one_delta_away_from_target() {
		let mut app = setup(Duration::from_millis(100));
		let agent = app
			.world_mut()
			.spawn((
				Transform {
					translation: Vec3::ZERO,
					scale: Vec3::splat(42.),
					rotation: Quat::from_rotation_y(PI),
				},
				ApplyMotion::from(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::new(1.099, 2., 3.),
				}),
			))
			.id();
		let ctrl = app
			.world_mut()
			.spawn((MotionControllerOf(agent), Transform::from_xyz(1., 2., 3.)))
			.id();

		app.update();

		assert_eq!(
			(
				Some(&ApplyMotion(CharacterMotion::Done)),
				None,
				Some(&Transform {
					translation: Vec3::new(1., 2., 3.),
					scale: Vec3::splat(42.),
					rotation: Quat::from_rotation_y(PI),
				}),
				Some(&OldTranslation(Vec3::new(1., 2., 3.))),
			),
			(
				app.world().entity(agent).get::<ApplyMotion>(),
				app.world().entity(agent).get::<IsInMotion>(),
				app.world().entity(agent).get::<Transform>(),
				app.world().entity(ctrl).get::<OldTranslation>(),
			)
		);
	}

	#[test]
	fn set_done_when_one_delta_away_from_target_accounting_for_speed() {
		let mut app = setup(Duration::from_millis(100));
		let agent = app
			.world_mut()
			.spawn((
				Transform {
					translation: Vec3::ZERO,
					scale: Vec3::splat(42.),
					rotation: Quat::from_rotation_y(PI),
				},
				ApplyMotion::from(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(2.)),
					target: Vec3::new(1.199, 2., 3.),
				}),
			))
			.id();
		let ctrl = app
			.world_mut()
			.spawn((MotionControllerOf(agent), Transform::from_xyz(1., 2., 3.)))
			.id();

		app.update();

		assert_eq!(
			(
				Some(&ApplyMotion(CharacterMotion::Done)),
				None,
				Some(&Transform {
					translation: Vec3::new(1., 2., 3.),
					scale: Vec3::splat(42.),
					rotation: Quat::from_rotation_y(PI),
				}),
				Some(&OldTranslation(Vec3::new(1., 2., 3.))),
			),
			(
				app.world().entity(agent).get::<ApplyMotion>(),
				app.world().entity(agent).get::<IsInMotion>(),
				app.world().entity(agent).get::<Transform>(),
				app.world().entity(ctrl).get::<OldTranslation>(),
			)
		);
	}

	#[test]
	fn set_done_when_one_delta_away_from_target_on_different_height() {
		let mut app = setup(Duration::from_millis(100));
		let agent = app
			.world_mut()
			.spawn((
				Transform {
					translation: Vec3::ZERO,
					scale: Vec3::splat(42.),
					rotation: Quat::from_rotation_y(PI),
				},
				ApplyMotion::from(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::new(1.099, 2. + ALLOWED_HEIGHT_DIFFERENCE, 3.),
				}),
			))
			.id();
		let ctrl = app
			.world_mut()
			.spawn((MotionControllerOf(agent), Transform::from_xyz(1., 2., 3.)))
			.id();

		app.update();

		assert_eq!(
			(
				Some(&ApplyMotion(CharacterMotion::Done)),
				None,
				Some(&Transform {
					translation: Vec3::new(1., 2., 3.),
					scale: Vec3::splat(42.),
					rotation: Quat::from_rotation_y(PI),
				}),
				Some(&OldTranslation(Vec3::new(1., 2., 3.))),
			),
			(
				app.world().entity(agent).get::<ApplyMotion>(),
				app.world().entity(agent).get::<IsInMotion>(),
				app.world().entity(agent).get::<Transform>(),
				app.world().entity(ctrl).get::<OldTranslation>(),
			)
		);
	}

	#[test]
	fn do_not_set_done_when_one_delta_away_from_target_on_large_different_height() {
		let mut app = setup(Duration::from_millis(100));
		let agent = app
			.world_mut()
			.spawn(ApplyMotion::from(CharacterMotion::ToTarget {
				speed: Speed(UnitsPerSecond::from(1.)),
				target: Vec3::new(1.099, 2. + ALLOWED_HEIGHT_DIFFERENCE + 0.1, 3.),
			}))
			.id();
		let ctrl = app
			.world_mut()
			.spawn((MotionControllerOf(agent), Transform::from_xyz(1., 2., 3.)))
			.id();

		app.update();

		assert_eq!(
			(
				Some(&ApplyMotion(CharacterMotion::ToTarget {
					speed: Speed(UnitsPerSecond::from(1.)),
					target: Vec3::new(1.099, 2. + ALLOWED_HEIGHT_DIFFERENCE + 0.1, 3.),
				})),
				Some(&IsInMotion),
				Some(&Transform::default()),
				Some(&OldTranslation::default()),
			),
			(
				app.world().entity(agent).get::<ApplyMotion>(),
				app.world().entity(agent).get::<IsInMotion>(),
				app.world().entity(agent).get::<Transform>(),
				app.world().entity(ctrl).get::<OldTranslation>(),
			)
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup(Duration::from_millis(100));
		let agent = app
			.world_mut()
			.spawn(ApplyMotion::from(CharacterMotion::ToTarget {
				speed: Speed(UnitsPerSecond::from(1.)),
				target: Vec3::ZERO,
			}))
			.id();
		app.world_mut()
			.spawn((MotionControllerOf(agent), Transform::default()));

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(agent).get::<IsChanged<ApplyMotion>>(),
		);
	}

	#[test]
	fn act_again_when_is_in_motion_present() {
		let mut app = setup(Duration::from_millis(100));
		let agent = app
			.world_mut()
			.spawn(ApplyMotion::from(CharacterMotion::ToTarget {
				speed: Speed(UnitsPerSecond::from(1.)),
				target: Vec3::ZERO,
			}))
			.id();
		app.world_mut()
			.spawn((MotionControllerOf(agent), Transform::default()));

		app.update();
		app.world_mut().entity_mut(agent).insert(IsInMotion);
		app.update();

		assert_eq!(
			Some(&IsChanged::TRUE),
			app.world().entity(agent).get::<IsChanged<ApplyMotion>>(),
		);
	}
}
