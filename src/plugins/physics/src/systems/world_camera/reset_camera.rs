use crate::components::world_camera::WorldCamera;
use bevy::prelude::*;

impl WorldCamera {
	pub(crate) fn reset_camera(mut cameras: Query<&mut Self>) {
		for mut cam in &mut cameras {
			if cam.mouse_hover.is_empty() {
				continue;
			}
			cam.mouse_hover.clear();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_physics::MouseHoversOver;
	use std::collections::HashMap;
	use testing::{IsChanged, SingleThreadedApp};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(
			Update,
			(WorldCamera::reset_camera, IsChanged::<WorldCamera>::detect).chain(),
		);

		app
	}

	#[test]
	fn reset_hover_in_camera() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(WorldCamera {
				mouse_hover: HashMap::from([(
					vec![],
					MouseHoversOver::Ground {
						point: Vec3::default(),
					},
				)]),
				..default()
			})
			.id();

		app.update();

		assert_eq!(
			Some(&WorldCamera::default()),
			app.world().entity(entity).get::<WorldCamera>(),
		);
	}

	#[test]
	fn leave_camera_unchanged_when_mouse_hover_is_empty() {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(WorldCamera {
				mouse_hover: HashMap::from([]),
				..default()
			})
			.id();

		app.update();
		app.update();

		assert_eq!(
			Some(&IsChanged::FALSE),
			app.world().entity(entity).get::<IsChanged<WorldCamera>>(),
		);
	}
}
