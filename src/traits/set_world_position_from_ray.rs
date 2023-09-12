mod mouse_event;

use bevy::prelude::*;

pub trait SetWorldPositionFromRay {
	fn set_world_position(&mut self, ray_cast: Ray);
}
