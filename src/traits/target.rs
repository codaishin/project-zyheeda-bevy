mod move_event;

use bevy::prelude::Vec3;

pub trait Target {
	fn target(&self) -> Vec3;
}
