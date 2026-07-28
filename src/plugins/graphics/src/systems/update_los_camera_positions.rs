use crate::components::los::LoSCameras;
use bevy::prelude::*;

impl LoSCameras {
	pub(crate) fn update_positions(
		cameras: Query<(&GlobalTransform, &Self), Changed<GlobalTransform>>,
		mut cameras_of: Query<&mut GlobalTransform, Without<Self>>,
	) {
		for (src, cameras) in cameras {
			for camera in cameras.iter() {
				let Ok(mut dst) = cameras_of.get_mut(camera) else {
					continue;
				};

				let transform = Transform {
					translation: src.translation(),
					rotation: dst.rotation(),
					..default()
				};

				*dst = GlobalTransform::from(transform);
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
			.spawn(GlobalTransform::from_xyz(1., 2., 3.))
			.id();
		let children = [
			app.world_mut().spawn(LoSCameraOf(parent)).id(),
			app.world_mut().spawn(LoSCameraOf(parent)).id(),
		];

		app.update();

		assert_eq!(
			[Some(&GlobalTransform::from_xyz(1., 2., 3.)); 2],
			app.world()
				.entity(children)
				.map(|e| e.get::<GlobalTransform>())
		);
	}

	#[test]
	fn preserve_camera_rotation() {
		let mut app = setup();
		let parent = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 2., 3.))
			.id();
		let children = [
			app.world_mut()
				.spawn((
					LoSCameraOf(parent),
					GlobalTransform::from(Transform::default().looking_to(Dir3::Y, Dir3::NEG_X)),
				))
				.id(),
			app.world_mut()
				.spawn((
					LoSCameraOf(parent),
					GlobalTransform::from(Transform::default().looking_to(Dir3::Y, Dir3::NEG_X)),
				))
				.id(),
		];

		app.update();

		assert_eq!(
			[Some(&GlobalTransform::from(
				Transform::from_xyz(1., 2., 3.).looking_to(Dir3::Y, Dir3::NEG_X)
			)); 2],
			app.world()
				.entity(children)
				.map(|e| e.get::<GlobalTransform>())
		);
	}

	#[test]
	fn act_only_once() {
		let mut app = setup();
		let parent = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 2., 3.))
			.id();
		let children = [
			app.world_mut().spawn(LoSCameraOf(parent)).id(),
			app.world_mut().spawn(LoSCameraOf(parent)).id(),
		];

		app.update();
		_ = app.world_mut().entity_mut(children).map(|mut e| {
			e.get_mut::<GlobalTransform>()
				.map(|mut t| *t = GlobalTransform::default())
		});
		app.update();

		assert_eq!(
			[Some(&GlobalTransform::default()); 2],
			app.world()
				.entity(children)
				.map(|e| e.get::<GlobalTransform>())
		);
	}

	#[test]
	fn act_again_if_transform_changed() {
		let mut app = setup();
		let parent = app
			.world_mut()
			.spawn(GlobalTransform::from_xyz(1., 2., 3.))
			.id();
		let children = [
			app.world_mut().spawn(LoSCameraOf(parent)).id(),
			app.world_mut().spawn(LoSCameraOf(parent)).id(),
		];

		app.update();
		_ = app.world_mut().entity_mut(children).map(|mut e| {
			e.get_mut::<GlobalTransform>()
				.map(|mut t| *t = GlobalTransform::default())
		});
		app.world_mut()
			.entity_mut(parent)
			.get_mut::<GlobalTransform>()
			.as_deref_mut();
		app.update();

		assert_eq!(
			[Some(&GlobalTransform::from_xyz(1., 2., 3.)); 2],
			app.world()
				.entity(children)
				.map(|e| e.get::<GlobalTransform>())
		);
	}
}
