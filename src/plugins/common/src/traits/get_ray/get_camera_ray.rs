use super::GetCamRay;
use bevy::{
	math::Ray3d,
	render::camera::Camera,
	transform::components::GlobalTransform,
	window::Window,
};

impl GetCamRay for Camera {
	fn get_ray(&self, camera_transform: &GlobalTransform, window: &Window) -> Option<Ray3d> {
		window
			.cursor_position()
			.and_then(|c| self.viewport_to_world(camera_transform, c))
	}
}
