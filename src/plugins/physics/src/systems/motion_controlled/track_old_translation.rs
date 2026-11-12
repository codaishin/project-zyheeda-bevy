use crate::components::{
	character_motion::IsInMotion,
	immobilized::Immobilized,
	motion_controller::{MotionControlled, OldTranslation},
};
use bevy::prelude::*;

impl MotionControlled {
	pub(crate) fn track_old_translation(
		controlled: Query<&Self, (With<IsInMotion>, Without<Immobilized>)>,
		mut controllers: Query<(&Transform, &mut OldTranslation)>,
	) {
		for controlled in controlled {
			let Ok((transform, mut old)) = controllers.get_mut(controlled.id()) else {
				continue;
			};

			*old = OldTranslation(transform.translation);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::motion_controller::MotionControllerOf;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, MotionControlled::track_old_translation);

		app
	}

	#[test]
	fn set_translation() {
		let mut app = setup();
		let controlled = app.world_mut().spawn(IsInMotion).id();
		let controller = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				MotionControllerOf::entity(controlled),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&OldTranslation(Vec3::new(1., 2., 3.))),
			app.world().entity(controller).get::<OldTranslation>(),
		);
	}

	#[test]
	fn do_nothing_if_not_in_motion() {
		let mut app = setup();
		let controlled = app.world_mut().spawn_empty().id();
		let controller = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				MotionControllerOf::entity(controlled),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&OldTranslation::default()),
			app.world().entity(controller).get::<OldTranslation>(),
		);
	}

	#[test]
	fn do_nothing_if_in_motion_and_immobilized() {
		let mut app = setup();
		let controlled = app.world_mut().spawn((IsInMotion, Immobilized)).id();
		let controller = app
			.world_mut()
			.spawn((
				Transform::from_xyz(1., 2., 3.),
				MotionControllerOf::entity(controlled),
			))
			.id();

		app.update();

		assert_eq!(
			Some(&OldTranslation::default()),
			app.world().entity(controller).get::<OldTranslation>(),
		);
	}
}
