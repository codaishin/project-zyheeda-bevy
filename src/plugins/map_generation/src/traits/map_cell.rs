use super::{
	map::{MapWindow, Neighbors, Tile},
	CellDistance,
	SourcePath,
};
use crate::{
	components::Corridor,
	map::{MapCell, Shape},
};
use bevy::math::primitives::Direction3d;
use common::traits::load_asset::Path;

impl SourcePath for MapCell {
	fn source_path() -> Path {
		Path::from("maps/map.txt")
	}
}

pub struct CellIsEmpty;

impl TryFrom<MapCell> for Path {
	type Error = CellIsEmpty;

	fn try_from(value: MapCell) -> Result<Self, Self::Error> {
		match value {
			MapCell::Corridor(_, Shape::Single) => corridor("single"),
			MapCell::Corridor(_, Shape::End) => corridor("end"),
			MapCell::Corridor(_, Shape::Straight) => corridor("straight"),
			MapCell::Corridor(_, Shape::Cross2) => corridor("corner"),
			MapCell::Corridor(_, Shape::Cross3) => corridor("t"),
			MapCell::Corridor(_, Shape::Cross4) => corridor("cross"),
			MapCell::Empty => empty_cell(),
		}
	}
}

fn corridor(suffix: &'static str) -> Result<Path, CellIsEmpty> {
	Ok(Path::from(format!(
		"{}{}.glb#Scene0",
		Corridor::MODEL_PATH_PREFIX,
		suffix
	)))
}

fn empty_cell() -> Result<Path, CellIsEmpty> {
	Err(CellIsEmpty)
}

impl From<MapCell> for Direction3d {
	fn from(value: MapCell) -> Self {
		match value {
			MapCell::Corridor(direction, _) => direction,
			MapCell::Empty => Direction3d::NEG_Z,
		}
	}
}

impl CellDistance for MapCell {
	const CELL_DISTANCE: f32 = 2.;
}

impl From<MapWindow> for MapCell {
	fn from(cross: MapWindow) -> Self {
		match cross {
			// Cross
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						up: Tile::Occupied,
						down: Tile::Occupied,
						right: Tile::Occupied,
						left: Tile::Occupied,
					},
			} => MapCell::Corridor(Direction3d::NEG_Z, Shape::Cross4),
			// T
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						up: Tile::Occupied,
						down: Tile::Occupied,
						left: Tile::Occupied,
						..
					},
			} => MapCell::Corridor(Direction3d::X, Shape::Cross3),
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						up: Tile::Occupied,
						left: Tile::Occupied,
						right: Tile::Occupied,
						..
					},
			} => MapCell::Corridor(Direction3d::Z, Shape::Cross3),
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						down: Tile::Occupied,
						left: Tile::Occupied,
						right: Tile::Occupied,
						..
					},
			} => MapCell::Corridor(Direction3d::NEG_Z, Shape::Cross3),
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						up: Tile::Occupied,
						down: Tile::Occupied,
						right: Tile::Occupied,
						..
					},
			} => MapCell::Corridor(Direction3d::NEG_X, Shape::Cross3),
			// Corners
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						up: Tile::Occupied,
						left: Tile::Occupied,
						..
					},
			} => MapCell::Corridor(Direction3d::X, Shape::Cross2),
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						up: Tile::Occupied,
						right: Tile::Occupied,
						..
					},
			} => MapCell::Corridor(Direction3d::Z, Shape::Cross2),
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						down: Tile::Occupied,
						left: Tile::Occupied,
						..
					},
			} => MapCell::Corridor(Direction3d::NEG_Z, Shape::Cross2),
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						down: Tile::Occupied,
						right: Tile::Occupied,
						..
					},
			} => MapCell::Corridor(Direction3d::NEG_X, Shape::Cross2),
			// Straights
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						right: Tile::Occupied,
						left: Tile::Occupied,
						..
					},
			} => MapCell::Corridor(Direction3d::NEG_X, Shape::Straight),
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						up: Tile::Occupied,
						down: Tile::Occupied,
						..
					},
			} => MapCell::Corridor(Direction3d::NEG_Z, Shape::Straight),
			// Ends
			MapWindow {
				focus: 'c',
				neighbors: Neighbors {
					right: Tile::Occupied,
					..
				},
			} => MapCell::Corridor(Direction3d::NEG_X, Shape::End),
			MapWindow {
				focus: 'c',
				neighbors: Neighbors {
					left: Tile::Occupied,
					..
				},
			} => MapCell::Corridor(Direction3d::X, Shape::End),
			MapWindow {
				focus: 'c',
				neighbors: Neighbors {
					up: Tile::Occupied, ..
				},
			} => MapCell::Corridor(Direction3d::Z, Shape::End),
			MapWindow {
				focus: 'c',
				neighbors: Neighbors {
					down: Tile::Occupied,
					..
				},
			} => MapCell::Corridor(Direction3d::NEG_Z, Shape::End),
			// Single
			MapWindow { focus: 'c', .. } => MapCell::Corridor(Direction3d::NEG_Z, Shape::Single),
			// None
			_ => MapCell::Empty,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;

	#[test]
	fn empty() {
		let cross = MapWindow {
			focus: 'x',
			..default()
		};

		assert_eq!(MapCell::Empty, MapCell::from(cross));
	}

	#[test]
	fn corridor_end_right() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				right: Tile::Occupied,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_X, Shape::End),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_end_left() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				left: Tile::Occupied,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::X, Shape::End),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_straight_horizontally() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				left: Tile::Occupied,
				right: Tile::Occupied,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_X, Shape::Straight),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_straight_vertically() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				left: Tile::Occupied,
				right: Tile::Occupied,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_X, Shape::Straight),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_left_upper_corner() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				down: Tile::Occupied,
				right: Tile::Occupied,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_X, Shape::Cross2),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_right_upper_corner() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				down: Tile::Occupied,
				left: Tile::Occupied,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_Z, Shape::Cross2),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_left_lower_corner() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				right: Tile::Occupied,
				up: Tile::Occupied,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::Z, Shape::Cross2),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_right_lower_corner() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				left: Tile::Occupied,
				up: Tile::Occupied,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::X, Shape::Cross2),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_t_down() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				left: Tile::Occupied,
				right: Tile::Occupied,
				down: Tile::Occupied,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_Z, Shape::Cross3),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_t_up() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				left: Tile::Occupied,
				right: Tile::Occupied,
				up: Tile::Occupied,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::Z, Shape::Cross3),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_t_right() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				down: Tile::Occupied,
				right: Tile::Occupied,
				up: Tile::Occupied,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_X, Shape::Cross3),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_t_left() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				down: Tile::Occupied,
				left: Tile::Occupied,
				up: Tile::Occupied,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::X, Shape::Cross3),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_cross() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				up: Tile::Occupied,
				down: Tile::Occupied,
				left: Tile::Occupied,
				right: Tile::Occupied,
			},
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_Z, Shape::Cross4),
			MapCell::from(cross)
		);
	}
}
