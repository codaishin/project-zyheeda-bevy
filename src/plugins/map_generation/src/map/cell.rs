use crate::{
	components::{
		floor_cell::FloorCell,
		quadrants::{
			CorridorFloor,
			CorridorFloorCornerInside,
			CorridorFloorCornerOutside,
			CorridorFloorForward,
			CorridorFloorLeft,
			CorridorWall,
			CorridorWallCornerInside,
			CorridorWallCornerOutside,
			CorridorWallCornerOutsideDiagonal,
			CorridorWallForward,
			CorridorWallLeft,
		},
		wall_cell::WallCell,
	},
	traits::{
		GridCellDistanceDefinition,
		SourcePath,
		insert_cell_components::InsertCellComponents,
		insert_cell_quadrant_components::{InsertCellQuadrantComponents, PatternMatches, Quadrant},
		is_walkable::IsWalkable,
	},
};
use bevy::prelude::*;
use common::traits::load_asset::Path;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone, TypePath)]
pub(crate) enum MapCell {
	CorridorFloor,
	CorridorWall,
}

impl MapCell {
	const MODEL_ASSET_CELL_WIDTH: f32 = 2.;
}

impl IsWalkable for MapCell {
	fn is_walkable(&self) -> bool {
		match self {
			MapCell::CorridorFloor => true,
			MapCell::CorridorWall => false,
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
			return MapCell::CorridorWall;
		};

		match symbol {
			'c' => MapCell::CorridorFloor,
			_ => MapCell::CorridorWall,
		}
	}
}

impl InsertCellComponents for MapCell {
	fn offset_height(&self) -> bool {
		match self {
			MapCell::CorridorFloor => false,
			MapCell::CorridorWall => true,
		}
	}

	fn insert_cell_components(&self, entity: &mut EntityCommands) {
		match self {
			MapCell::CorridorFloor => entity.insert(FloorCell),
			MapCell::CorridorWall => entity.insert(WallCell),
		};
	}
}

impl InsertCellQuadrantComponents for MapCell {
	fn insert_cell_quadrant_components(
		&self,
		entity: &mut EntityCommands,
		differences: HashSet<Quadrant>,
	) {
		match self {
			// Corridor Floor
			MapCell::CorridorFloor if differences.matches(CORNER_INNER) => {
				entity.insert(CorridorFloorCornerInside);
			}
			MapCell::CorridorFloor if differences.matches(CORNER_OUTER) => {
				entity.insert(CorridorFloorCornerOutside);
			}
			MapCell::CorridorFloor if differences.matches(CORNER_OUTER_DIAGONAL) => {
				entity.insert(CorridorFloorCornerOutside);
			}
			MapCell::CorridorFloor if differences.contains(&Quadrant::Forward) => {
				entity.insert(CorridorFloorForward);
			}
			MapCell::CorridorFloor if differences.contains(&Quadrant::Left) => {
				entity.insert(CorridorFloorLeft);
			}
			MapCell::CorridorFloor => {
				entity.insert(CorridorFloor);
			}
			// Corridor Wall
			MapCell::CorridorWall if differences.matches(CORNER_INNER) => {
				entity.insert(CorridorWallCornerInside);
			}
			MapCell::CorridorWall if differences.matches(CORNER_OUTER) => {
				entity.insert(CorridorWallCornerOutside);
			}
			MapCell::CorridorWall if differences.matches(CORNER_OUTER_DIAGONAL) => {
				entity.insert(CorridorWallCornerOutsideDiagonal);
			}
			MapCell::CorridorWall if differences.contains(&Quadrant::Forward) => {
				entity.insert(CorridorWallForward);
			}
			MapCell::CorridorWall if differences.contains(&Quadrant::Left) => {
				entity.insert(CorridorWallLeft);
			}
			MapCell::CorridorWall => {
				entity.insert(CorridorWall);
			}
		};
	}
}

const CORNER_INNER: [Quadrant; 1] = [Quadrant::Diagonal];
const CORNER_OUTER: [Quadrant; 3] = [Quadrant::Left, Quadrant::Diagonal, Quadrant::Forward];
const CORNER_OUTER_DIAGONAL: [Quadrant; 2] = [Quadrant::Left, Quadrant::Forward];

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn is_walkable() {
		let cell = MapCell::CorridorFloor;

		assert!(cell.is_walkable());
	}

	#[test]
	fn is_not_walkable() {
		let cell = MapCell::CorridorWall;

		assert!(!cell.is_walkable());
	}

	#[test]
	fn new_empty_cell() {
		let symbol = Some('c');

		let cell = MapCell::from(symbol);

		assert_eq!(MapCell::CorridorFloor, cell);
	}

	#[test]
	fn new_wall_cell() {
		let symbol = Some('は');

		let cell = MapCell::from(symbol);

		assert_eq!(MapCell::CorridorWall, cell);
	}

	#[test]
	fn new_wall_cell_from_none() {
		let symbol = None;

		let cell = MapCell::from(symbol);

		assert_eq!(MapCell::CorridorWall, cell);
	}
}
