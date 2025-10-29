use super::GetCamRay;
use bevy::prelude::*;

impl GetCamRay<Window> for Camera {
	fn get_ray(&self, camera_transform: &GlobalTransform, window: &Window) -> Option<Ray3d> {
		window
			.cursor_position()
			.and_then(|c| self.viewport_to_world(camera_transform, c).ok())
	}
}
