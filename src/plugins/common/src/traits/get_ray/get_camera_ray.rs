use super::GetCamRay;
use bevy::{
	prelude::{Camera, GlobalTransform, Ray},
	window::Window,
};

impl GetCamRay for Camera {
	fn get_ray(&self, camera_transform: &GlobalTransform, window: &Window) -> Option<Ray> {
		window
			.cursor_position()
			.and_then(|c| self.viewport_to_world(camera_transform, c))
	}
}
