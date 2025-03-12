use crate::{
	components::Corridor,
	tools::model_data::ModelData,
	traits::{
		GridCellDistanceDefinition,
		SourcePath,
		is_walkable::IsWalkable,
		map::{Direction, MapWindow, Tile},
	},
};
use bevy::prelude::*;
use common::traits::load_asset::Path;
use std::collections::{HashMap, HashSet};

#[derive(Debug, PartialEq, Clone, TypePath)]
pub(crate) enum MapCell {
	Floor,
	Wall {
		walls: HashMap<Direction, Wall>,
		details: HashSet<Direction>,
	},
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

const DIRECTIONS: [Direction; 4] = [
	Direction::NegZ,
	Direction::NegZ.rotated_right(1),
	Direction::NegZ.rotated_right(2),
	Direction::NegZ.rotated_right(3),
];

impl From<MapWindow> for MapCell {
	fn from(value: MapWindow) -> Self {
		if value.focus == 'c' {
			return MapCell::Floor;
		}

		let neighbors = value.neighbors;
		let mut walls = HashMap::default();
		let mut details = HashSet::default();

		for dir in DIRECTIONS {
			match (neighbors[dir], neighbors[dir.rotated_right(1)]) {
				(Tile::Walkable, Tile::Walkable) => {
					walls.insert(dir, Wall::Corner);
					details.insert(dir);
				}
				(Tile::Walkable, Tile::NotWalkable) => {
					walls.insert(dir, Wall::Half);
					details.insert(dir);
				}
				(Tile::NotWalkable, Tile::Walkable) => {
					walls.insert(dir, Wall::HalfRotated);
				}
				(Tile::NotWalkable, Tile::NotWalkable) => {}
			};
		}

		MapCell::Wall { walls, details }
	}
}

impl From<&MapCell> for ModelData {
	fn from(cell: &MapCell) -> Self {
		match cell {
			MapCell::Floor => model_data!([("floor", Dir3::NEG_Z)]),
			MapCell::Wall { walls, details } => {
				let floor = [("wall_floor", Direction::NegZ)].into_iter();
				let walls = walls.iter().map(|(dir, wall)| (<&str>::from(wall), *dir));
				let details = details.iter().map(|dir| ("wall_details", *dir));
				model_data!(floor.chain(walls).chain(details))
			}
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Wall {
	Half,
	HalfRotated,
	Corner,
}

impl From<&Wall> for &'static str {
	fn from(value: &Wall) -> Self {
		match value {
			Wall::Half => "half_wall",
			Wall::HalfRotated => "half_wall_rotated",
			Wall::Corner => "wall_corner",
		}
	}
}

macro_rules! model_data {
	($raw:expr) => {
		ModelData::from_iter($raw.map(|(suffix, dir)| {
			(
				format!("{}{}.glb#Scene0", Corridor::MODEL_PATH_PREFIX, suffix),
				Dir3::from(dir),
			)
		}))
	};
}
use model_data;

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{assert_eq_model_data, traits::map::Neighbors};
	use test_case::test_case;

	const NOT_C: char = 'は';

	#[test]
	fn is_walkable() {
		let cell = MapCell::Floor;

		assert!(cell.is_walkable());
	}

	#[test]
	fn is_not_walkable() {
		let cell = MapCell::Wall {
			walls: HashMap::default(),
			details: HashSet::default(),
		};

		assert!(!cell.is_walkable());
	}

	#[test]
	fn new_empty_cell() {
		let window = MapWindow {
			focus: 'c',
			neighbors: Neighbors::default(),
		};

		let cell = MapCell::from(window);

		assert_eq!(MapCell::Floor, cell);
	}

	#[test]
	fn new_wall_cell() {
		let window = MapWindow {
			focus: NOT_C,
			neighbors: Neighbors::default(),
		};

		let cell = MapCell::from(window);

		assert_eq!(
			MapCell::Wall {
				walls: HashMap::default(),
				details: HashSet::default(),
			},
			cell
		);
	}

	#[test_case(Direction::NegZ; "neg z")]
	#[test_case(Direction::NegX; "neg x")]
	#[test_case(Direction::Z; "z")]
	#[test_case(Direction::X; "x")]
	fn new_wall_cell_with_corner(dir: Direction) {
		let window = MapWindow {
			focus: NOT_C,
			neighbors: Neighbors::default()
				.with(dir, Tile::Walkable)
				.with(dir.rotated_right(1), Tile::Walkable),
		};

		let cell = MapCell::from(window);

		assert_eq!(
			MapCell::Wall {
				walls: HashMap::from([
					(dir.rotated_left(), Wall::HalfRotated),
					(dir, Wall::Corner),
					(dir.rotated_right(1), Wall::Half),
				]),
				details: HashSet::from([dir, dir.rotated_right(1)]),
			},
			cell
		);
	}

	#[test_case(Direction::NegZ; "neg z")]
	#[test_case(Direction::NegX; "neg x")]
	#[test_case(Direction::Z; "z")]
	#[test_case(Direction::X; "x")]
	fn new_wall_cell_wall(dir: Direction) {
		let window = MapWindow {
			focus: NOT_C,
			neighbors: Neighbors::default().with(dir, Tile::Walkable),
		};

		let cell = MapCell::from(window);

		assert_eq!(
			MapCell::Wall {
				walls: HashMap::from([(dir.rotated_left(), Wall::HalfRotated), (dir, Wall::Half)]),
				details: HashSet::from([dir]),
			},
			cell
		);
	}

	#[test]
	fn floor_cell_model_data() {
		let cell = MapCell::Floor;

		assert_eq_model_data!(
			model_data!([("floor", Dir3::NEG_Z)]),
			ModelData::from(&cell)
		);
	}

	#[test]
	fn wall_cell_floor_model_data() {
		let cell = MapCell::Wall {
			walls: HashMap::default(),
			details: HashSet::default(),
		};

		assert_eq_model_data!(
			model_data!([("wall_floor", Dir3::NEG_Z)]),
			ModelData::from(&cell)
		);
	}

	#[test]
	fn wall_cell_wall_model_data() {
		let cell = MapCell::Wall {
			walls: HashMap::from([
				(Direction::Z, Wall::Corner),
				(Direction::NegX, Wall::Half),
				(Direction::NegZ, Wall::HalfRotated),
			]),
			details: HashSet::default(),
		};

		assert_eq_model_data!(
			model_data!([
				("wall_floor", Dir3::NEG_Z),
				("wall_corner", Dir3::Z),
				("half_wall", Dir3::NEG_X),
				("half_wall_rotated", Dir3::NEG_Z),
			]),
			ModelData::from(&cell)
		);
	}

	#[test]
	fn wall_cell_wall_details() {
		let cell = MapCell::Wall {
			walls: HashMap::default(),
			details: HashSet::from([Direction::X, Direction::NegZ]),
		};

		assert_eq_model_data!(
			model_data!([
				("wall_floor", Dir3::NEG_Z),
				("wall_details", Dir3::X),
				("wall_details", Dir3::NEG_Z),
			]),
			ModelData::from(&cell)
		);
	}
}
