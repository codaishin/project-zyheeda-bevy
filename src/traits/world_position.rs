use bevy::prelude::Vec3;

pub trait GetWorldPosition {
	fn get_world_position(&self) -> Option<Vec3>;
}
