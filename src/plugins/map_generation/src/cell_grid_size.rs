use crate::grid_graph::grid_context::CellCount;
use macros::new_valid;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct CellGridSize {
	pub(crate) x: CellCount,
	pub(crate) z: CellCount,
}

impl CellGridSize {
	pub(crate) const DEFAULT: Self = Self {
		x: new_valid!(CellCount, 1),
		z: new_valid!(CellCount, 1),
	};
}

impl Default for CellGridSize {
	fn default() -> Self {
		Self::DEFAULT
	}
}
