use bevy::{math::Vec2, window::Window};

pub trait MousePosition {
	fn mouse_position(&self) -> Option<Vec2>;
}

impl MousePosition for Window {
	fn mouse_position(&self) -> Option<Vec2> {
		self.cursor_position()
	}
}
