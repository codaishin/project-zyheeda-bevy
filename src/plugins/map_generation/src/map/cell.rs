use crate::traits::{GridCellDistanceDefinition, SourcePath, is_walkable::IsWalkable};
use bevy::prelude::*;
use common::traits::load_asset::Path;

#[derive(Debug, PartialEq, Clone, TypePath)]
pub(crate) enum MapCell {
	Floor,
	Wall,
}

impl MapCell {
	const MODEL_ASSET_CELL_WIDTH: f32 = 2.;
}

impl IsWalkable for MapCell {
	fn is_walkable(&self) -> bool {
		match self {
			MapCell::Floor => true,
			MapCell::Wall { .. } => false,
		}
	}
}

impl SourcePath for MapCell {
	fn source_path() -> Path {
		Path::from("maps/map.txt")
	}
}

impl GridCellDistanceDefinition for MapCell {
	const CELL_DISTANCE: f32 = MapCell::MODEL_ASSET_CELL_WIDTH;
}

impl From<Option<char>> for MapCell {
	fn from(symbol: Option<char>) -> Self {
		let Some(symbol) = symbol else {
			return MapCell::Wall;
		};

		match symbol {
			'c' => MapCell::Floor,
			_ => MapCell::Wall,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn is_walkable() {
		let cell = MapCell::Floor;

		assert!(cell.is_walkable());
	}

	#[test]
	fn is_not_walkable() {
		let cell = MapCell::Wall;

		assert!(!cell.is_walkable());
	}

	#[test]
	fn new_empty_cell() {
		let symbol = Some('c');

		let cell = MapCell::from(symbol);

		assert_eq!(MapCell::Floor, cell);
	}

	#[test]
	fn new_wall_cell() {
		let symbol = Some('は');

		let cell = MapCell::from(symbol);

		assert_eq!(MapCell::Wall, cell);
	}

	#[test]
	fn new_wall_cell_from_none() {
		let symbol = None;

		let cell = MapCell::from(symbol);

		assert_eq!(MapCell::Wall, cell);
	}
}
