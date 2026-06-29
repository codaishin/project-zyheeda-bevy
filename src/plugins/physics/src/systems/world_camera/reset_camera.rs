use crate::resources::world_camera::WorldCamera;
use bevy::prelude::*;

impl WorldCamera {
	pub(crate) fn reset_camera(mut world_camera: ResMut<Self>) {
		if world_camera.mouse_hover.is_empty() {
			return;
		}

		world_camera.mouse_hover.clear();
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::traits::handles_physics::{MouseHover, MouseHoversOver};
	use std::collections::HashMap;
	use testing::SingleThreadedApp;

	#[derive(Resource, Debug, PartialEq)]
	struct _Changed(bool);

	fn setup(world_camera: WorldCamera) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(world_camera);
		app.add_systems(
			Update,
			(
				WorldCamera::reset_camera,
				|cam: Res<WorldCamera>, mut c: Commands| {
					c.insert_resource(_Changed(cam.is_changed()));
				},
			)
				.chain(),
		);

		app
	}

	#[test]
	fn reset_hover_in_camera() {
		let mut app = setup(WorldCamera {
			mouse_hover: HashMap::from([(
				MouseHover::default(),
				MouseHoversOver::Point(Vec3::default()),
			)]),
			..default()
		});

		app.update();

		assert_eq!(
			&WorldCamera::default(),
			app.world().resource::<WorldCamera>(),
		);
	}

	#[test]
	fn leave_camera_unchanged_when_mouse_hover_is_empty() {
		let mut app = setup(WorldCamera {
			mouse_hover: HashMap::from([]),
			..default()
		});

		app.update();
		app.update();

		assert_eq!(&_Changed(false), app.world().resource::<_Changed>());
	}
}
