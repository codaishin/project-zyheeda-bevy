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
pub(crate) enum Corridor {
	CorridorFloor,
	CorridorWall,
}

impl Corridor {
	const MODEL_ASSET_CELL_WIDTH: f32 = 2.;
}

impl IsWalkable for Corridor {
	fn is_walkable(&self) -> bool {
		match self {
			Corridor::CorridorFloor => true,
			Corridor::CorridorWall => false,
		}
	}
}

impl SourcePath for Corridor {
	fn source_path() -> Path {
		Path::from("maps/map.txt")
	}
}

impl GridCellDistanceDefinition for Corridor {
	const CELL_DISTANCE: f32 = Corridor::MODEL_ASSET_CELL_WIDTH;
}

impl From<Option<char>> for Corridor {
	fn from(symbol: Option<char>) -> Self {
		let Some(symbol) = symbol else {
			return Corridor::CorridorWall;
		};

		match symbol {
			'c' => Corridor::CorridorFloor,
			_ => Corridor::CorridorWall,
		}
	}
}

impl InsertCellComponents for Corridor {
	fn offset_height(&self) -> bool {
		match self {
			Corridor::CorridorFloor => false,
			Corridor::CorridorWall => true,
		}
	}

	fn insert_cell_components(&self, entity: &mut EntityCommands) {
		match self {
			Corridor::CorridorFloor => entity.insert(FloorCell),
			Corridor::CorridorWall => entity.insert(WallCell),
		};
	}
}

impl InsertCellQuadrantComponents for Corridor {
	fn insert_cell_quadrant_components(
		&self,
		entity: &mut EntityCommands,
		differences: HashSet<Quadrant>,
	) {
		match self {
			// Corridor Floor
			Corridor::CorridorFloor if differences.matches(CORNER_INNER) => {
				entity.insert(CorridorFloorCornerInside);
			}
			Corridor::CorridorFloor if differences.matches(CORNER_OUTER) => {
				entity.insert(CorridorFloorCornerOutside);
			}
			Corridor::CorridorFloor if differences.matches(CORNER_OUTER_DIAGONAL) => {
				entity.insert(CorridorFloorCornerOutside);
			}
			Corridor::CorridorFloor if differences.contains(&Quadrant::Forward) => {
				entity.insert(CorridorFloorForward);
			}
			Corridor::CorridorFloor if differences.contains(&Quadrant::Left) => {
				entity.insert(CorridorFloorLeft);
			}
			Corridor::CorridorFloor => {
				entity.insert(CorridorFloor);
			}
			// Corridor Wall
			Corridor::CorridorWall if differences.matches(CORNER_INNER) => {
				entity.insert(CorridorWallCornerInside);
			}
			Corridor::CorridorWall if differences.matches(CORNER_OUTER) => {
				entity.insert(CorridorWallCornerOutside);
			}
			Corridor::CorridorWall if differences.matches(CORNER_OUTER_DIAGONAL) => {
				entity.insert(CorridorWallCornerOutsideDiagonal);
			}
			Corridor::CorridorWall if differences.contains(&Quadrant::Forward) => {
				entity.insert(CorridorWallForward);
			}
			Corridor::CorridorWall if differences.contains(&Quadrant::Left) => {
				entity.insert(CorridorWallLeft);
			}
			Corridor::CorridorWall => {
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
		let cell = Corridor::CorridorFloor;

		assert!(cell.is_walkable());
	}

	#[test]
	fn is_not_walkable() {
		let cell = Corridor::CorridorWall;

		assert!(!cell.is_walkable());
	}

	#[test]
	fn new_empty_cell() {
		let symbol = Some('c');

		let cell = Corridor::from(symbol);

		assert_eq!(Corridor::CorridorFloor, cell);
	}

	#[test]
	fn new_wall_cell() {
		let symbol = Some('は');

		let cell = Corridor::from(symbol);

		assert_eq!(Corridor::CorridorWall, cell);
	}

	#[test]
	fn new_wall_cell_from_none() {
		let symbol = None;

		let cell = Corridor::from(symbol);

		assert_eq!(Corridor::CorridorWall, cell);
	}
}
