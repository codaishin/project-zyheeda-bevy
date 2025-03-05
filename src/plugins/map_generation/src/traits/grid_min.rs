use bevy::prelude::*;

pub(crate) trait GridMin {
	fn grid_min(&self) -> Vec3;
}
