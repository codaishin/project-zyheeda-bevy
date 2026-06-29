use crate::system_params::camera::CameraContextMut;
use bevy::prelude::*;
use common::traits::handles_graphics::ScreenPosition;

impl ScreenPosition for CameraContextMut<'_> {
	fn screen_position(&self, translation: Vec3) -> Option<Vec2> {
		let entity = self.world_camera.ui_cam?;
		let (transform, camera) = self.cameras.get(entity).ok()?;

		camera.world_to_viewport(transform, translation).ok()
	}
}
