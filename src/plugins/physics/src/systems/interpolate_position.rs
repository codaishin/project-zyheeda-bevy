use crate::components::motion_controller::MotionControllerOf;
use bevy::prelude::*;

impl MotionControllerOf {
	pub(crate) fn interpolate_position(
		In(OverstepFraction(overstep)): In<OverstepFraction>,
		controllers: Query<(&Self, &Transform)>,
		mut transforms: Query<&mut Transform, Without<Self>>,
	) {
		for (Self(controlled), transform) in controllers {
			let Ok(mut controlled) = transforms.get_mut(*controlled) else {
				continue;
			};

			controlled.translation = controlled.translation.lerp(transform.translation, overstep);
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
	use testing::SingleThreadedApp;

	fn setup(overstep: OverstepFraction) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(move || overstep).pipe(MotionControllerOf::interpolate_position),
		);

		app
	}

	#[test]
	fn overstep_one() {
		let mut app = setup(OverstepFraction(1.));
		let entity = app.world_mut().spawn(Transform::from_xyz(1., 2., 3.)).id();
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
		let entity = app.world_mut().spawn(Transform::from_xyz(1., 2., 3.)).id();
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
		let entity = app.world_mut().spawn(Transform::from_xyz(1., 2., 3.)).id();
		app.world_mut()
			.spawn((MotionControllerOf(entity), Transform::from_xyz(1., 2., 5.)));

		app.update();

		assert_eq!(
			Some(&Transform::from_xyz(1., 2., 4.)),
			app.world().entity(entity).get::<Transform>(),
		);
	}
}
