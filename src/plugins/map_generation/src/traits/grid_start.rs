use bevy::prelude::*;

pub(crate) trait GridStart {
	fn grid_min(&self) -> Vec3;
}
