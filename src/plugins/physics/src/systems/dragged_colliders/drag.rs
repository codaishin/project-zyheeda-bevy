use crate::components::dragged_colliders::{DragColliderWith, DraggedColliders};
use bevy::prelude::*;

impl DraggedColliders {
	pub(crate) fn drag(
		anchored: Query<(&DragColliderWith, &mut Transform)>,
		anchors: Query<&GlobalTransform>,
	) {
		for (DragColliderWith(anchor), mut transform) in anchored {
			let Ok(anchor) = anchors.get(*anchor) else {
				continue;
			};

			*transform = Transform::from(*anchor);
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, DraggedColliders::drag);

		app
	}

	#[test]
	fn update_transform_of_dragged() {
		let mut app = setup();
		let global = GlobalTransform::from(
			Transform::from_xyz(1., 2., 3.)
				.with_scale(Vec3::splat(2.))
				.looking_to(Dir3::X, Dir3::Y),
		);
		let anchor = app.world_mut().spawn(global).id();
		let anchored = app.world_mut().spawn(DragColliderWith(anchor)).id();

		app.update();

		assert_eq!(
			Some(Transform::from(global)),
			app.world().entity(anchored).get::<Transform>().copied(),
		);
	}
}
