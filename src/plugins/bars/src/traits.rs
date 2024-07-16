pub mod health;

use bevy::{
	color::Color,
	math::{Vec2, Vec3},
	render::camera::Camera,
	transform::components::GlobalTransform,
};

pub trait UIBarUpdate<T> {
	fn update(&mut self, value: &T);
}

pub trait UIBarColors {
	fn background_color() -> Color;
	fn foreground_color() -> Color;
}

pub trait GetScreenPosition {
	fn get_screen_position(
		&self,
		camera_transform: &GlobalTransform,
		world_position: Vec3,
	) -> Option<Vec2>;
}

impl GetScreenPosition for Camera {
	fn get_screen_position(
		&self,
		camera_transform: &GlobalTransform,
		world_position: Vec3,
	) -> Option<Vec2> {
		self.world_to_viewport(camera_transform, world_position)
	}
}
