use bevy::prelude::*;

pub(crate) trait GridStart {
	fn grid_start(&self) -> Vec3;
}
