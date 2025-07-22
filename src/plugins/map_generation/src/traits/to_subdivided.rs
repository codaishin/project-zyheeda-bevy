use crate::grid_graph::grid_context::DividedToZero;

pub trait ToSubdivided: Sized {
	fn to_subdivided(&self, subdivisions: u8) -> Result<Self, DividedToZero>;
}
