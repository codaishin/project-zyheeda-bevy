pub mod get_camera_ray;

use bevy::{math::Ray3d, transform::components::GlobalTransform, window::Window};

pub trait GetCamRay {
	fn get_ray(&self, camera_transform: &GlobalTransform, window: &Window) -> Option<Ray3d>;
}
