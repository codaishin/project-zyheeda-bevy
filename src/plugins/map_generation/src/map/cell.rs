use super::Shape;
use crate::{
	components::Corridor,
	tools::model_data::ModelData,
	traits::{
		GridCellDistanceDefinition,
		SourcePath,
		is_walkable::IsWalkable,
		map::{MapWindow, Neighbors, Tile},
	},
};
use bevy::{math::Dir3, prelude::*};
use common::traits::load_asset::Path;

#[derive(Debug, PartialEq, Clone, Copy, TypePath)]
pub(crate) enum MapCell {
	Corridor(Dir3, Shape),
	Empty,
}

impl IsWalkable for MapCell {
	fn is_walkable(&self) -> bool {
		match self {
			MapCell::Corridor(..) => true,
			MapCell::Empty => false,
		}
	}
}

impl SourcePath for MapCell {
	fn source_path() -> Path {
		Path::from("maps/map.txt")
	}
}

impl From<&MapCell> for ModelData {
	fn from(value: &MapCell) -> Self {
		match value {
			MapCell::Corridor(dir, Shape::Single) => corridor([(dir, "single")]),
			MapCell::Corridor(dir, Shape::End) => corridor([(dir, "end")]),
			MapCell::Corridor(dir, Shape::Straight) => corridor([(dir, "straight")]),
			MapCell::Corridor(dir, Shape::Cross2) => corridor([(dir, "corner")]),
			MapCell::Corridor(dir, Shape::Cross3) => corridor([(dir, "t")]),
			MapCell::Corridor(dir, Shape::Cross4) => corridor([(dir, "cross")]),
			MapCell::Empty => corridor([]),
		}
	}
}

fn corridor<const N: usize>(suffixes: [(&Dir3, &str); N]) -> ModelData {
	ModelData::from(suffixes.map(|(dir, suffix)| {
		(
			format!("{}{}.glb#Scene0", Corridor::MODEL_PATH_PREFIX, suffix),
			*dir,
		)
	}))
}

impl GridCellDistanceDefinition for MapCell {
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
						up: Tile::Walkable,
						down: Tile::Walkable,
						right: Tile::Walkable,
						left: Tile::Walkable,
					},
			} => MapCell::Corridor(Dir3::Z, Shape::Cross4),
			// T
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						up: Tile::Walkable,
						down: Tile::Walkable,
						left: Tile::Walkable,
						..
					},
			} => MapCell::Corridor(Dir3::NEG_X, Shape::Cross3),
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						up: Tile::Walkable,
						left: Tile::Walkable,
						right: Tile::Walkable,
						..
					},
			} => MapCell::Corridor(Dir3::NEG_Z, Shape::Cross3),
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						down: Tile::Walkable,
						left: Tile::Walkable,
						right: Tile::Walkable,
						..
					},
			} => MapCell::Corridor(Dir3::Z, Shape::Cross3),
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						up: Tile::Walkable,
						down: Tile::Walkable,
						right: Tile::Walkable,
						..
					},
			} => MapCell::Corridor(Dir3::X, Shape::Cross3),
			// Corners
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						up: Tile::Walkable,
						left: Tile::Walkable,
						..
					},
			} => MapCell::Corridor(Dir3::NEG_X, Shape::Cross2),
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						up: Tile::Walkable,
						right: Tile::Walkable,
						..
					},
			} => MapCell::Corridor(Dir3::NEG_Z, Shape::Cross2),
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						down: Tile::Walkable,
						left: Tile::Walkable,
						..
					},
			} => MapCell::Corridor(Dir3::Z, Shape::Cross2),
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						down: Tile::Walkable,
						right: Tile::Walkable,
						..
					},
			} => MapCell::Corridor(Dir3::X, Shape::Cross2),
			// Straights
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						right: Tile::Walkable,
						left: Tile::Walkable,
						..
					},
			} => MapCell::Corridor(Dir3::X, Shape::Straight),
			MapWindow {
				focus: 'c',
				neighbors:
					Neighbors {
						up: Tile::Walkable,
						down: Tile::Walkable,
						..
					},
			} => MapCell::Corridor(Dir3::Z, Shape::Straight),
			// Ends
			MapWindow {
				focus: 'c',
				neighbors: Neighbors {
					right: Tile::Walkable,
					..
				},
			} => MapCell::Corridor(Dir3::X, Shape::End),
			MapWindow {
				focus: 'c',
				neighbors: Neighbors {
					left: Tile::Walkable,
					..
				},
			} => MapCell::Corridor(Dir3::NEG_X, Shape::End),
			MapWindow {
				focus: 'c',
				neighbors: Neighbors {
					up: Tile::Walkable, ..
				},
			} => MapCell::Corridor(Dir3::NEG_Z, Shape::End),
			MapWindow {
				focus: 'c',
				neighbors: Neighbors {
					down: Tile::Walkable,
					..
				},
			} => MapCell::Corridor(Dir3::Z, Shape::End),
			// Single
			MapWindow { focus: 'c', .. } => MapCell::Corridor(Dir3::Z, Shape::Single),
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
				right: Tile::Walkable,
				..default()
			},
		};

		assert_eq!(MapCell::Corridor(Dir3::X, Shape::End), MapCell::from(cross));
	}

	#[test]
	fn corridor_end_left() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				left: Tile::Walkable,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Dir3::NEG_X, Shape::End),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_straight_horizontally() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				left: Tile::Walkable,
				right: Tile::Walkable,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Dir3::X, Shape::Straight),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_straight_vertically() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				left: Tile::Walkable,
				right: Tile::Walkable,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Dir3::X, Shape::Straight),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_left_upper_corner() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				down: Tile::Walkable,
				right: Tile::Walkable,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Dir3::X, Shape::Cross2),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_right_upper_corner() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				down: Tile::Walkable,
				left: Tile::Walkable,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Dir3::Z, Shape::Cross2),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_left_lower_corner() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				right: Tile::Walkable,
				up: Tile::Walkable,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Dir3::NEG_Z, Shape::Cross2),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_right_lower_corner() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				left: Tile::Walkable,
				up: Tile::Walkable,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Dir3::NEG_X, Shape::Cross2),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_t_down() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				left: Tile::Walkable,
				right: Tile::Walkable,
				down: Tile::Walkable,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Dir3::Z, Shape::Cross3),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_t_up() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				left: Tile::Walkable,
				right: Tile::Walkable,
				up: Tile::Walkable,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Dir3::NEG_Z, Shape::Cross3),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_t_right() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				down: Tile::Walkable,
				right: Tile::Walkable,
				up: Tile::Walkable,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Dir3::X, Shape::Cross3),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_t_left() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				down: Tile::Walkable,
				left: Tile::Walkable,
				up: Tile::Walkable,
				..default()
			},
		};

		assert_eq!(
			MapCell::Corridor(Dir3::NEG_X, Shape::Cross3),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_cross() {
		let cross = MapWindow {
			focus: 'c',
			neighbors: Neighbors {
				up: Tile::Walkable,
				down: Tile::Walkable,
				left: Tile::Walkable,
				right: Tile::Walkable,
			},
		};

		assert_eq!(
			MapCell::Corridor(Dir3::Z, Shape::Cross4),
			MapCell::from(cross)
		);
	}

	#[test]
	fn is_walkable() {
		let cells = [MapCell::Empty, MapCell::Corridor(Dir3::Z, Shape::Straight)];

		assert_eq!([false, true], cells.map(|c| c.is_walkable()));
	}
}
