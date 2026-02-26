use common::traits::handles_map_generation::GroundPosition;

pub(crate) trait GridMin {
	fn grid_min(&self) -> GroundPosition;
}
