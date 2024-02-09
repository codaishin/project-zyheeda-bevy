use bevy::{
	math::{Vec2, Vec3},
	render::camera::Camera,
	transform::components::GlobalTransform,
};

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
