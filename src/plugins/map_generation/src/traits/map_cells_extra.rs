use crate::{cell_grid_size::CellGridSize, components::map::cells::CellGrid};
use common::traits::thread_safe::ThreadSafe;

pub(crate) trait MapCellsExtra: Sized {
	type TExtra: for<'a> From<&'a CellGridDefinition<Self>> + ThreadSafe;
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct CellGridDefinition<TCell> {
	pub(crate) size: CellGridSize,
	pub(crate) cells: CellGrid<TCell>,
}

impl<TCell> Default for CellGridDefinition<TCell> {
	fn default() -> Self {
		Self {
			size: CellGridSize::default(),
			cells: CellGrid::default(),
		}
	}
}

impl<TCell> From<&CellGridDefinition<TCell>> for () {
	fn from(_: &CellGridDefinition<TCell>) -> Self {}
}
