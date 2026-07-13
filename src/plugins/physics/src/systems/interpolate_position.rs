use crate::components::motion_controller::{MotionController, OldTranslation};
use bevy::prelude::*;

impl MotionController {
	pub(crate) fn interpolate_position(
		In(OverstepFraction(overstep)): In<OverstepFraction>,
		controlled: Query<(&mut Transform, &OldTranslation, &Self)>,
		controllers: Query<&Transform, Without<Self>>,
	) {
		for (mut transform, OldTranslation(old_translation), ctrl) in controlled {
			let Ok(ctrl_transform) = controllers.get(ctrl.get()) else {
				continue;
			};

			if &transform.translation == old_translation {
				continue;
			}

			transform.translation = old_translation.lerp(ctrl_transform.translation, overstep);
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct OverstepFraction(f32);

impl OverstepFraction {
	pub(crate) fn fixed(fixed_time: Res<Time<Fixed>>) -> OverstepFraction {
		OverstepFraction(fixed_time.overstep_fraction())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::motion_controller::MotionControllerOf;
	use testing::{IsChanged, SingleThreadedApp};

	fn setup(overstep: OverstepFraction) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(
				(move || overstep).pipe(MotionController::interpolate_position),
				IsChanged::<Transform>::detect,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn overstep_one() {
		let mut app = setup(OverstepFraction(1.));
		let entity = app
			.world_mut()
			.spawn(OldTranslation(Vec3::new(1., 2., 3.)))
			.id();
		app.world_mut()
			.spawn((MotionControllerOf(entity), Transform::from_xyz(1., 2., 5.)));

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(1., 2., 5.)),
			app.world().entity(entity).get::<Transform>(),
		);
	}

	#[test]
	fn overstep_zero() {
		let mut app = setup(OverstepFraction(0.));
		let entity = app
			.world_mut()
			.spawn(OldTranslation(Vec3::new(1., 2., 3.)))
			.id();
		app.world_mut()
			.spawn((MotionControllerOf(entity), Transform::from_xyz(1., 2., 5.)));

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(1., 2., 3.)),
			app.world().entity(entity).get::<Transform>(),
		);
	}

	#[test]
	fn overstep_interpolate_half() {
		let mut app = setup(OverstepFraction(0.5));
		let entity = app
			.world_mut()
			.spawn(OldTranslation(Vec3::new(1., 2., 3.)))
			.id();
		app.world_mut()
			.spawn((MotionControllerOf(entity), Transform::from_xyz(1., 2., 5.)));

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(1., 2., 4.)),
			app.world().entity(entity).get::<Transform>(),
		);
	}

	#[test]
	fn do_nothing_if_old_translation_is_new_translation() {
		let mut app = setup(OverstepFraction(1.));
		let entity = app
			.world_mut()
			.spawn(OldTranslation(Vec3::new(1., 2., 3.)))
			.id();
		app.world_mut()
			.spawn((MotionControllerOf(entity), Transform::from_xyz(1., 2., 5.)));

		app.update();
		app.world_mut()
			.entity_mut(entity)
			.insert(OldTranslation(Vec3::new(1., 2., 5.)));
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<Transform>>(),
		);
	}
}
