use super::GetRayFromCamera;
use crate::tools::Tools;
use bevy::{
	prelude::{Camera, GlobalTransform, Ray},
	window::Window,
};

impl GetRayFromCamera for Tools {
	fn get_ray(
		camera: &Camera,
		camera_transform: &GlobalTransform,
		window: &Window,
	) -> Option<Ray> {
		window
			.cursor_position()
			.and_then(|c| camera.viewport_to_world(camera_transform, c))
	}
}
