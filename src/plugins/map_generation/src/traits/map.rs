use super::MapAsset;
use crate::map_loader::{Cell, Map};

impl MapAsset<Cell> for Map {
	const CELL_DISTANCE: f32 = 2.;

	fn cells(&self) -> Vec<Vec<Cell>> {
		self.0 .0.clone()
	}
}
