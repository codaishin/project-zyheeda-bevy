use crate::map::Map;
use bevy::{math::Dir3, reflect::TypePath};
use std::ops::{Index, IndexMut};

impl<TCell: From<MapWindow> + TypePath + Sync + Send> From<String> for Map<TCell> {
	fn from(value: String) -> Self {
		let lines: Vec<String> = value
			.split('\n')
			.map(strip_white_spaces)
			.filter(|line| non_empty(line))
			.collect();

		let map = lines
			.iter()
			.enumerate()
			.map(parse_via_map_window(&lines))
			.collect();

		Self(map)
	}
}

#[derive(Default, Debug, PartialEq)]
pub(crate) struct Neighbors {
	pub up: Tile,
	pub down: Tile,
	pub left: Tile,
	pub right: Tile,
}

#[cfg(test)]
impl Neighbors {
	pub(crate) fn with(mut self, direction: Direction, tile: Tile) -> Self {
		self[direction] = tile;

		self
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub(crate) enum Direction {
	X,
	NegX,
	Z,
	NegZ,
}

impl Direction {
	pub(crate) const fn rotated_right(self, times: u8) -> Self {
		if times == 0 {
			return self;
		}

		if times > 1 {
			return self.rotated_right(1).rotated_right(times - 1);
		}

		match self {
			Direction::NegZ => Direction::NegX,
			Direction::NegX => Direction::Z,
			Direction::Z => Direction::X,
			Direction::X => Direction::NegZ,
		}
	}

	#[cfg(test)]
	pub(crate) const fn rotated_left(self) -> Self {
		match self {
			Direction::NegZ => Direction::X,
			Direction::NegX => Direction::NegZ,
			Direction::Z => Direction::NegX,
			Direction::X => Direction::Z,
		}
	}
}

impl Index<Direction> for Neighbors {
	type Output = Tile;

	fn index(&self, index: Direction) -> &Self::Output {
		match index {
			Direction::X => &self.right,
			Direction::NegX => &self.left,
			Direction::Z => &self.down,
			Direction::NegZ => &self.up,
		}
	}
}

impl IndexMut<Direction> for Neighbors {
	fn index_mut(&mut self, index: Direction) -> &mut Self::Output {
		match index {
			Direction::X => &mut self.right,
			Direction::NegX => &mut self.left,
			Direction::Z => &mut self.down,
			Direction::NegZ => &mut self.up,
		}
	}
}

impl From<Direction> for Dir3 {
	fn from(dir: Direction) -> Self {
		match dir {
			Direction::X => Dir3::X,
			Direction::NegX => Dir3::NEG_X,
			Direction::Z => Dir3::Z,
			Direction::NegZ => Dir3::NEG_Z,
		}
	}
}

#[derive(Default, Debug, PartialEq)]
pub(crate) struct MapWindow {
	pub focus: char,
	pub neighbors: Neighbors,
}

struct MapCoordinates {
	horizontal: usize,
	vertical: usize,
}

struct MapValues<'a> {
	cells: &'a Vec<Vec<char>>,
	focus: char,
}

fn parse_via_map_window<TCell: From<MapWindow>>(
	lines: &'_ [String],
) -> impl FnMut((usize, &String)) -> Vec<TCell> + '_ {
	let cells = lines.iter().map(|l| l.chars().collect()).collect();
	move |(line_i, line)| {
		line.chars()
			.enumerate()
			.map(map_window(line_i, &cells))
			.map(TCell::from)
			.collect()
	}
}

fn map_window(
	line_i: usize,
	cells: &Vec<Vec<char>>,
) -> impl FnMut((usize, char)) -> MapWindow + '_ {
	move |(char_i, char)| {
		MapWindow::new(
			MapValues { focus: char, cells },
			MapCoordinates {
				horizontal: char_i,
				vertical: line_i,
			},
		)
	}
}

fn strip_white_spaces(line: &str) -> String {
	line.chars()
		.filter(|c| !c.is_whitespace())
		.collect::<String>()
}

fn non_empty(line: &str) -> bool {
	!line.is_empty()
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub(crate) enum Tile {
	#[default]
	NotWalkable,
	Walkable,
}

impl<'a> From<Option<&'a char>> for Tile {
	fn from(value: Option<&'a char>) -> Self {
		match value {
			Some('c') => Tile::Walkable,
			_ => Tile::NotWalkable,
		}
	}
}

impl MapWindow {
	fn new(values: MapValues, coordinates: MapCoordinates) -> Self {
		let MapValues { cells, focus } = values;
		let MapCoordinates {
			vertical,
			horizontal,
		} = coordinates;
		let neighbors = Neighbors {
			up: Tile::from(
				vertical
					.checked_sub(1)
					.and_then(|vertical| cells[vertical].get(horizontal)),
			),
			down: Tile::from(
				vertical
					.checked_add(1)
					.filter(|vertical| vertical < &cells.len())
					.and_then(|vertical| cells[vertical].get(horizontal)),
			),
			left: Tile::from(
				horizontal
					.checked_sub(1)
					.and_then(|horizontal| cells[vertical].get(horizontal)),
			),
			right: Tile::from(
				horizontal
					.checked_add(1)
					.and_then(|horizontal| cells[vertical].get(horizontal)),
			),
		};
		Self { focus, neighbors }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;
	use test_case::test_case;

	#[derive(TypePath, Debug, PartialEq)]
	struct _Cell(MapWindow);

	impl From<MapWindow> for _Cell {
		fn from(value: MapWindow) -> Self {
			_Cell(value)
		}
	}

	#[test]
	fn single() {
		let raw = "x".to_string();
		let map = Map::<_Cell>::from(raw);

		assert_eq!(
			Map(vec![vec![_Cell(MapWindow {
				focus: 'x',
				..default()
			})]]),
			map
		);
	}

	#[test]
	fn double() {
		let raw = "cx".to_string();
		let map = Map::<_Cell>::from(raw);

		assert_eq!(
			Map(vec![vec![
				_Cell(MapWindow {
					focus: 'c',
					neighbors: Neighbors {
						right: Tile::NotWalkable,
						..default()
					}
				}),
				_Cell(MapWindow {
					focus: 'x',
					neighbors: Neighbors {
						left: Tile::Walkable,
						..default()
					}
				})
			]]),
			map
		);
	}

	#[test]
	fn skip_white_spaces() {
		let raw = "x c".to_string();
		let map = Map::<_Cell>::from(raw);

		assert_eq!(
			Map(vec![vec![
				_Cell(MapWindow {
					focus: 'x',
					neighbors: Neighbors {
						right: Tile::Walkable,
						..default()
					}
				}),
				_Cell(MapWindow {
					focus: 'c',
					neighbors: Neighbors {
						left: Tile::NotWalkable,
						..default()
					}
				})
			]]),
			map
		);
	}

	#[test]
	fn process_multiple_lines() {
		let raw = "
			xct
			erj
			lpn
		"
		.to_string();
		let map = Map::from(raw);

		assert_eq!(
			Map(vec![
				vec![
					_Cell(MapWindow {
						focus: 'x',
						neighbors: Neighbors {
							right: Tile::Walkable,
							..default()
						}
					}),
					_Cell(MapWindow {
						focus: 'c',
						neighbors: Neighbors::default()
					}),
					_Cell(MapWindow {
						focus: 't',
						neighbors: Neighbors {
							left: Tile::Walkable,
							..default()
						}
					}),
				],
				vec![
					_Cell(MapWindow {
						focus: 'e',
						neighbors: Neighbors::default()
					}),
					_Cell(MapWindow {
						focus: 'r',
						neighbors: Neighbors {
							up: Tile::Walkable,
							..default()
						}
					}),
					_Cell(MapWindow {
						focus: 'j',
						neighbors: Neighbors::default()
					}),
				],
				vec![
					_Cell(MapWindow {
						focus: 'l',
						neighbors: Neighbors::default()
					}),
					_Cell(MapWindow {
						focus: 'p',
						neighbors: Neighbors::default()
					}),
					_Cell(MapWindow {
						focus: 'n',
						neighbors: Neighbors::default()
					}),
				]
			]),
			map
		);
	}

	#[test]
	fn neighbors_index() {
		let neighbors = Neighbors {
			up: Tile::Walkable,
			left: Tile::Walkable,
			..default()
		};

		assert_eq!(
			[
				Tile::Walkable,
				Tile::Walkable,
				Tile::NotWalkable,
				Tile::NotWalkable,
			],
			[
				neighbors[Direction::NegZ],
				neighbors[Direction::NegX],
				neighbors[Direction::Z],
				neighbors[Direction::X],
			]
		);
	}

	#[test]
	fn neighbors_index_mut() {
		let mut neighbors = Neighbors {
			up: Tile::Walkable,
			left: Tile::Walkable,
			..default()
		};

		neighbors[Direction::NegZ] = Tile::NotWalkable;
		neighbors[Direction::NegX] = Tile::NotWalkable;
		neighbors[Direction::Z] = Tile::Walkable;
		neighbors[Direction::X] = Tile::Walkable;

		assert_eq!(
			[
				Tile::NotWalkable,
				Tile::NotWalkable,
				Tile::Walkable,
				Tile::Walkable,
			],
			[
				neighbors[Direction::NegZ],
				neighbors[Direction::NegX],
				neighbors[Direction::Z],
				neighbors[Direction::X],
			]
		);
	}

	#[test_case(Direction::NegZ, Dir3::NEG_Z; "neg z")]
	#[test_case(Direction::NegX, Dir3::NEG_X; "neg x")]
	#[test_case(Direction::Z, Dir3::Z; "z")]
	#[test_case(Direction::X, Dir3::X; "x")]
	fn dir3_from_direction(value: Direction, result: Dir3) {
		assert_eq!(result, Dir3::from(value));
	}

	#[test_case(Direction::NegZ, Direction::NegX, 1; "neg z once")]
	#[test_case(Direction::NegX, Direction::Z, 1; "neg x once")]
	#[test_case(Direction::Z, Direction::X, 1; "z once")]
	#[test_case(Direction::X, Direction::NegZ, 1; "x once")]
	#[test_case(Direction::NegZ, Direction::NegZ, 0; "neg z zero")]
	#[test_case(Direction::NegZ, Direction::Z, 2; "neg z twice")]
	#[test_case(Direction::NegZ, Direction::X, 3; "neg z thrice")]
	fn rotate_right(value: Direction, result: Direction, times: u8) {
		assert_eq!(result, value.rotated_right(times));
	}

	#[test_case(Direction::NegZ, Direction::X; "neg z")]
	#[test_case(Direction::NegX, Direction::NegZ; "neg x")]
	#[test_case(Direction::Z, Direction::NegX; "z")]
	#[test_case(Direction::X, Direction::Z; "x")]
	fn rotate_left(value: Direction, result: Direction) {
		assert_eq!(result, value.rotated_left());
	}
}
