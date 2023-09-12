mod mouse_event;

use bevy::prelude::*;

pub trait GetTarget {
	fn get_target(&self) -> Option<Vec3>;
}
