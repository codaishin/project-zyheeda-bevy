use crate::{resources::CamRay, traits::get_ray::GetRayFromCamera};
use bevy::{
	ecs::system::{Commands, Query},
	render::camera::Camera,
	transform::components::GlobalTransform,
	window::Window,
};

pub fn set_cam_ray<TTools: GetRayFromCamera>(
	mut commands: Commands,
	camera: Query<(&Camera, &GlobalTransform)>,
	window: Query<&Window>,
) {
	let (camera, camera_transform) = camera.single();
	let window = window.single();

	commands.insert_resource(CamRay(TTools::get_ray(camera, camera_transform, window)));
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		core_pipeline::core_3d::Camera3dBundle,
		math::{Ray, Vec3},
		utils::default,
	};
	use mockall::mock;

	macro_rules! setup_mock {
		($struct_name:ident) => {
			mock! {
				$struct_name {}
				impl GetRayFromCamera for $struct_name{
					fn get_ray(
						_camera: &Camera,
						_camera_transform: &GlobalTransform,
						_window: &Window,
					) -> Option<Ray> {}
				}
			}
		};
	}

	fn setup<TGetRay: GetRayFromCamera + 'static>() -> App {
		let mut app = App::new();

		app.world.spawn(Camera3dBundle {
			camera: Camera {
				order: 42,
				..default()
			},
			global_transform: GlobalTransform::from_xyz(4., 3., 2.),
			..default()
		});
		app.world.spawn(Window {
			title: "Window".to_owned(),
			..default()
		});
		app.add_systems(Update, set_cam_ray::<TGetRay>);

		app
	}

	setup_mock!(SomeRay);

	#[test]
	fn add_ray() {
		let mut app = setup::<MockSomeRay>();

		let get_ray = MockSomeRay::get_ray_context();
		get_ray.expect().return_const(Ray {
			origin: Vec3::new(1., 2., 3.),
			direction: Vec3::new(4., 5., 6.),
		});

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

	setup_mock!(NoRay);

	#[test]
	fn add_none_ray() {
		let mut app = setup::<MockNoRay>();

		let get_ray = MockNoRay::get_ray_context();
		get_ray.expect().return_const(None);

		app.update();

		let cam_ray = app.world.resource::<CamRay>();

		assert!(cam_ray.0.is_none());
	}

	setup_mock!(CheckCalls);

	#[test]
	fn call_get_ray_with_proper_components() {
		let mut app = setup::<MockCheckCalls>();

		let get_ray = MockCheckCalls::get_ray_context();
		get_ray
			.expect()
			.withf(|cam, cam_transform, window| {
				// using some fields for structs that do not implement PartialEq
				*cam_transform == GlobalTransform::from_xyz(4., 3., 2.)
					&& cam.order == 42 && window.title == "Window"
			})
			.times(1)
			.return_const(None);

		app.update();
	}
}
