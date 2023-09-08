use bevy::prelude::Vec3;

pub trait WorldPosition {
	fn get_world_position(&self) -> Option<Vec3>;
}
