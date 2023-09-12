pub mod get_camera_ray;

use bevy::{
	prelude::{Camera, GlobalTransform, Ray},
	window::Window,
};

pub trait GetRayFromCamera {
	fn get_ray(camera: &Camera, camera_transform: &GlobalTransform, window: &Window)
		-> Option<Ray>;
}
