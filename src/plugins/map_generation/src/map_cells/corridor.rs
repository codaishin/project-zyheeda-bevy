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
	map_cells::parsed_color::ParsedColor,
	resources::color_lookup::ColorLookup,
	systems::color_lookup::load_images::ColorLookupAssetPath,
	traits::{
		GridCellDistanceDefinition,
		insert_cell_components::InsertCellComponents,
		insert_cell_quadrant_components::{InsertCellQuadrantComponents, PatternMatches, Quadrant},
		is_walkable::IsWalkable,
	},
};
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone, TypePath, Default)]
pub(crate) enum Corridor {
	#[default]
	Wall,
	Floor,
}

impl Corridor {
	const MODEL_ASSET_CELL_WIDTH: f32 = 2.;
}

impl IsWalkable for Corridor {
	fn is_walkable(&self) -> bool {
		match self {
			Corridor::Floor => true,
			Corridor::Wall => false,
		}
	}
}

impl GridCellDistanceDefinition for Corridor {
	const CELL_DISTANCE: f32 = Corridor::MODEL_ASSET_CELL_WIDTH;
}

impl From<Option<char>> for Corridor {
	fn from(symbol: Option<char>) -> Self {
		let Some(symbol) = symbol else {
			return Corridor::Wall;
		};

		match symbol {
			'c' => Corridor::Floor,
			_ => Corridor::Wall,
		}
	}
}

impl From<(ParsedColor, ColorLookup<Corridor>)> for Corridor {
	fn from((parsed, lookup): (ParsedColor, ColorLookup<Corridor>)) -> Self {
		if matches!(parsed.color(), Some(color) if color == &lookup.floor) {
			return Corridor::Floor;
		}

		Corridor::Wall
	}
}

impl ColorLookupAssetPath for Corridor {
	const LOOKUP_ROOT: &str = "maps/lookup/corridor";
}

impl InsertCellComponents for Corridor {
	fn offset_height(&self) -> bool {
		match self {
			Corridor::Floor => false,
			Corridor::Wall => true,
		}
	}

	fn insert_cell_components(&self, entity: &mut EntityCommands) {
		match self {
			Corridor::Floor => entity.insert(FloorCell),
			Corridor::Wall => entity.insert(WallCell),
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
			Corridor::Floor if differences.matches(CORNER_INNER) => {
				entity.insert(CorridorFloorCornerInside);
			}
			Corridor::Floor if differences.matches(CORNER_OUTER) => {
				entity.insert(CorridorFloorCornerOutside);
			}
			Corridor::Floor if differences.matches(CORNER_OUTER_DIAGONAL) => {
				entity.insert(CorridorFloorCornerOutside);
			}
			Corridor::Floor if differences.contains(&Quadrant::Forward) => {
				entity.insert(CorridorFloorForward);
			}
			Corridor::Floor if differences.contains(&Quadrant::Left) => {
				entity.insert(CorridorFloorLeft);
			}
			Corridor::Floor => {
				entity.insert(CorridorFloor);
			}
			// Corridor Wall
			Corridor::Wall if differences.matches(CORNER_INNER) => {
				entity.insert(CorridorWallCornerInside);
			}
			Corridor::Wall if differences.matches(CORNER_OUTER) => {
				entity.insert(CorridorWallCornerOutside);
			}
			Corridor::Wall if differences.matches(CORNER_OUTER_DIAGONAL) => {
				entity.insert(CorridorWallCornerOutsideDiagonal);
			}
			Corridor::Wall if differences.contains(&Quadrant::Forward) => {
				entity.insert(CorridorWallForward);
			}
			Corridor::Wall if differences.contains(&Quadrant::Left) => {
				entity.insert(CorridorWallLeft);
			}
			Corridor::Wall => {
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
		let cell = Corridor::Floor;

		assert!(cell.is_walkable());
	}

	#[test]
	fn is_not_walkable() {
		let cell = Corridor::Wall;

		assert!(!cell.is_walkable());
	}

	#[test]
	fn new_empty_cell() {
		let symbol = Some('c');

		let cell = Corridor::from(symbol);

		assert_eq!(Corridor::Floor, cell);
	}

	#[test]
	fn new_wall_cell() {
		let symbol = Some('は');

		let cell = Corridor::from(symbol);

		assert_eq!(Corridor::Wall, cell);
	}

	#[test]
	fn new_wall_cell_from_none() {
		let symbol = None;

		let cell = Corridor::from(symbol);

		assert_eq!(Corridor::Wall, cell);
	}
}
