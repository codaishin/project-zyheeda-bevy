use crate::{resources::CamRay, traits::get_ray::GetCamRay};
use bevy::{
	ecs::{
		component::Component,
		system::{Commands, Query},
	},
	transform::components::GlobalTransform,
	window::Window,
};

pub(crate) fn set_cam_ray<TCamera: GetCamRay + Component>(
	mut commands: Commands,
	camera: Query<(&TCamera, &GlobalTransform)>,
	window: Query<&Window>,
) {
	let (camera, camera_transform) = camera.single();
	let window = window.single();

	commands.insert_resource(CamRay(camera.get_ray(camera_transform, window)));
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		math::{Ray, Vec3},
		utils::default,
	};
	use mockall::automock;

	#[derive(Component, Default)]
	struct _Camera {
		mock: Mock_Camera,
	}

	#[automock]
	impl GetCamRay for _Camera {
		fn get_ray(&self, camera_transform: &GlobalTransform, window: &Window) -> Option<Ray> {
			self.mock.get_ray(camera_transform, window)
		}
	}

	fn setup(cam: _Camera) -> App {
		let mut app = App::new();

		app.world
			.spawn((cam, GlobalTransform::from_xyz(4., 3., 2.)));
		app.world.spawn(Window {
			title: "Window".to_owned(),
			..default()
		});
		app.add_systems(Update, set_cam_ray::<_Camera>);

		app
	}

	#[test]
	fn add_ray() {
		let mut cam = _Camera::default();
		cam.mock.expect_get_ray().return_const(Ray {
			origin: Vec3::new(1., 2., 3.),
			direction: Vec3::new(4., 5., 6.),
		});
		let mut app = setup(cam);

		app.update();

		let cam_ray = app.world.resource::<CamRay>();

		assert_eq!(
			Some(Ray {
				origin: Vec3::new(1., 2., 3.),
				direction: Vec3::new(4., 5., 6.),
			}),
			cam_ray.0
		);
	}

	#[test]
	fn add_none_ray() {
		let mut cam = _Camera::default();
		cam.mock.expect_get_ray().return_const(None);
		let mut app = setup(cam);

		app.update();

		let cam_ray = app.world.resource::<CamRay>();

		assert!(cam_ray.0.is_none());
	}

	#[test]
	fn call_get_ray_with_proper_components() {
		let mut cam = _Camera::default();
		cam.mock
			.expect_get_ray()
			.withf(|cam_transform, window| {
				{
					*cam_transform == GlobalTransform::from_xyz(4., 3., 2.)
						&& window.title == "Window"
				}
			})
			.times(1)
			.return_const(None);

		let mut app = setup(cam);

		app.update();
	}
}
