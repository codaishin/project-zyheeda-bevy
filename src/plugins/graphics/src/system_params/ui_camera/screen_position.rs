use crate::system_params::ui_camera::UiCameraContextMut;
use bevy::prelude::*;
use common::traits::handles_graphics::ScreenPosition;

impl ScreenPosition for UiCameraContextMut<'_> {
	fn screen_position(&self, translation: Vec3) -> Option<Vec2> {
		self.camera
			.world_to_viewport(self.transform, translation)
			.ok()
	}
}
