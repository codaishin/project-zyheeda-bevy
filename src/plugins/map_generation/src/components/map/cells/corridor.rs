use crate::{
	components::{
		floor_cell::FloorCell,
		map::cells::parsed_color::ParsedColor,
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
	grid_graph::grid_context::CellDistance,
	resources::map::color_lookup::MapColorLookup,
	systems::map_color_lookup::load_images::ColorLookupAssetPath,
	traits::{
		GridCellDistanceDefinition,
		insert_cell_components::InsertCellComponents,
		insert_cell_quadrant_components::{InsertCellQuadrantComponents, PatternMatches, Quadrant},
		is_walkable::IsWalkable,
		parse_map_image::ParseMapImage,
	},
};
use bevy::prelude::*;
use common::errors::Unreachable;
use macros::new_valid;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Clone, TypePath, Default)]
pub(crate) enum Corridor {
	#[default]
	Wall,
	Floor,
}

impl Corridor {
	const MODEL_ASSET_CELL_WIDTH: CellDistance = new_valid!(CellDistance, 2.);
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
	const CELL_DISTANCE: CellDistance = Corridor::MODEL_ASSET_CELL_WIDTH;
}

impl ParseMapImage<ParsedColor> for Corridor {
	type TParseError = Unreachable;
	type TLookup = MapColorLookup<Corridor>;

	fn try_parse(
		image: &ParsedColor,
		lookup: &MapColorLookup<Corridor>,
	) -> Result<Self, Unreachable> {
		if matches!(image.color(), Some(color) if color == &lookup.floor) {
			return Ok(Corridor::Floor);
		}

		Ok(Corridor::Wall)
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
		different_quadrants: HashSet<Quadrant>,
	) {
		match self {
			// Corridor Floor
			Corridor::Floor if different_quadrants.matches(CORNER_INNER) => {
				entity.insert(CorridorFloorCornerInside);
			}
			Corridor::Floor if different_quadrants.matches(CORNER_OUTER) => {
				entity.insert(CorridorFloorCornerOutside);
			}
			Corridor::Floor if different_quadrants.matches(CORNER_OUTER_DIAGONAL) => {
				entity.insert(CorridorFloorCornerOutside);
			}
			Corridor::Floor if different_quadrants.contains(&Quadrant::Forward) => {
				entity.insert(CorridorFloorForward);
			}
			Corridor::Floor if different_quadrants.contains(&Quadrant::Left) => {
				entity.insert(CorridorFloorLeft);
			}
			Corridor::Floor => {
				entity.insert(CorridorFloor);
			}
			// Corridor Wall
			Corridor::Wall if different_quadrants.matches(CORNER_INNER) => {
				entity.insert(CorridorWallCornerInside);
			}
			Corridor::Wall if different_quadrants.matches(CORNER_OUTER) => {
				entity.insert(CorridorWallCornerOutside);
			}
			Corridor::Wall if different_quadrants.matches(CORNER_OUTER_DIAGONAL) => {
				entity.insert(CorridorWallCornerOutsideDiagonal);
			}
			Corridor::Wall if different_quadrants.contains(&Quadrant::Forward) => {
				entity.insert(CorridorWallForward);
			}
			Corridor::Wall if different_quadrants.contains(&Quadrant::Left) => {
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
}
