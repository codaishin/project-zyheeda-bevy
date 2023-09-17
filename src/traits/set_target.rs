mod simple_movement;

use bevy::prelude::Vec3;

pub trait SetTarget {
	fn set_target(&mut self, target: Option<Vec3>);
}
