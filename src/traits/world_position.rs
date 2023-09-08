use bevy::prelude::*;

pub trait GetWorldPosition {
	fn get_world_position(&self) -> Option<Vec3>;
}

pub trait SetWorldPositionFromRay {
	fn set_world_position(&mut self, ray_cast: Ray);
}
