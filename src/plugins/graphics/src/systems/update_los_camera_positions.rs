use crate::components::los::LoSCameras;
use bevy::prelude::*;

impl LoSCameras {
	pub(crate) fn update_positions(
		cameras: Query<(&Transform, &Self), Changed<Transform>>,
		mut cameras_of: Query<&mut Transform, Without<Self>>,
	) {
		for (src, cameras) in cameras {
			for camera in cameras.iter() {
				let Ok(mut dst) = cameras_of.get_mut(camera) else {
					continue;
				};

				dst.translation = src.translation;
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::los::LoSCameraOf;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, LoSCameras::update_positions);

		app
	}

	#[test]
	fn set_position() {
		let mut app = setup();
		let parent = app
			.world_mut()
			.spawn(Transform::from_xyz(1., 2., 3.).looking_to(Dir3::X, Dir3::Y))
			.id();
		let children = [
			app.world_mut().spawn(LoSCameraOf(parent)).id(),
			app.world_mut().spawn(LoSCameraOf(parent)).id(),
		];

		app.update();

		assert_eq!(
			[Some(&Transform::from_xyz(1., 2., 3.)); 2],
			app.world().entity(children).map(|e| e.get::<Transform>())
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let parent = app
			.world_mut()
			.spawn(Transform::from_xyz(1., 2., 3.).looking_to(Dir3::X, Dir3::Y))
			.id();
		let children = [
			app.world_mut().spawn(LoSCameraOf(parent)).id(),
			app.world_mut().spawn(LoSCameraOf(parent)).id(),
		];

		app.update();
		_ = app.world_mut().entity_mut(children).map(|mut e| {
			e.get_mut::<Transform>()
				.map(|mut t| t.translation = Vec3::ZERO)
		});
		app.update();

		assert_eq!(
			[Some(&Transform::default()); 2],
			app.world().entity(children).map(|e| e.get::<Transform>())
		);
	}

	#[test]
	fn act_again_if_transform_changed() {
		let mut app = setup();
		let parent = app
			.world_mut()
			.spawn(Transform::from_xyz(1., 2., 3.).looking_to(Dir3::X, Dir3::Y))
			.id();
		let children = [
			app.world_mut().spawn(LoSCameraOf(parent)).id(),
			app.world_mut().spawn(LoSCameraOf(parent)).id(),
		];

		app.update();
		_ = app.world_mut().entity_mut(children).map(|mut e| {
			e.get_mut::<Transform>()
				.map(|mut t| t.translation = Vec3::ZERO)
		});
		app.world_mut()
			.entity_mut(parent)
			.get_mut::<Transform>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			[Some(&Transform::from_xyz(1., 2., 3.)); 2],
			app.world().entity(children).map(|e| e.get::<Transform>())
		);
	}
}
