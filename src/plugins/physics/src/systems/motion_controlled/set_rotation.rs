use crate::components::{motion_controller::MotionControlled, skill::Skill};
use bevy::prelude::*;

impl MotionControlled<Skill> {
	pub(crate) fn set_rotation(
		controlled: Query<(&mut Transform, &Self)>,
		transforms: Query<&Transform, Without<Self>>,
	) {
		for (mut transform, controlled) in controlled {
			let Ok(controller) = transforms.get(controlled.controller()) else {
				continue;
			};

			transform.rotation = controller.rotation;
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

		app.add_systems(Update, MotionControlled::set_rotation);

		app
	}

	#[test]
	fn set_rotation() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Transform::default()).id();
		app.world_mut().spawn((
			MotionControllerOf::skill(entity),
			Transform::from_rotation(Quat::from_rotation_x(0.5)),
		));

		app.update();

		assert_eq!(
			Some(&Transform::from_rotation(Quat::from_rotation_x(0.5))),
			app.world().entity(entity).get::<Transform>(),
		);
	}

	#[test]
	fn do_not_set_translation_or_scale() {
		let mut app = setup();
		let entity = app.world_mut().spawn(Transform::default()).id();
		app.world_mut().spawn((
			MotionControllerOf::skill(entity),
			Transform::from_rotation(Quat::from_rotation_x(0.5))
				.with_scale(Vec3::splat(2.))
				.with_translation(Vec3::new(1., 2., 3.)),
		));

		app.update();

		assert_eq!(
			Some(&Transform::from_rotation(Quat::from_rotation_x(0.5))),
			app.world().entity(entity).get::<Transform>(),
		);
	}
}
