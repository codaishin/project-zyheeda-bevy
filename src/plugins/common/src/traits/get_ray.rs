pub mod get_camera_ray;

use bevy::{
	prelude::{GlobalTransform, Ray},
	window::Window,
};

pub trait GetCamRay {
	fn get_ray(&self, camera_transform: &GlobalTransform, window: &Window) -> Option<Ray>;
}
