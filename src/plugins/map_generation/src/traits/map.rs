use crate::map::Map;
use bevy::{math::Dir3, reflect::TypePath};
use std::ops::Index;

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

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum NeighborDirection {
	X,
	NegX,
	Z,
	NegZ,
}

impl Index<NeighborDirection> for Neighbors {
	type Output = Tile;

	fn index(&self, index: NeighborDirection) -> &Self::Output {
		match index {
			NeighborDirection::X => &self.right,
			NeighborDirection::NegX => &self.left,
			NeighborDirection::Z => &self.down,
			NeighborDirection::NegZ => &self.up,
		}
	}
}

impl From<NeighborDirection> for Dir3 {
	fn from(dir: NeighborDirection) -> Self {
		match dir {
			NeighborDirection::X => Dir3::X,
			NeighborDirection::NegX => Dir3::NEG_X,
			NeighborDirection::Z => Dir3::Z,
			NeighborDirection::NegZ => Dir3::NEG_Z,
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
	fn neighbors() {
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
				neighbors[NeighborDirection::NegZ],
				neighbors[NeighborDirection::NegX],
				neighbors[NeighborDirection::Z],
				neighbors[NeighborDirection::X],
			]
		);
	}

	#[test]
	fn dir3_from_neighbor_direction() {
		assert_eq!(
			[Dir3::NEG_Z, Dir3::NEG_X, Dir3::Z, Dir3::X],
			[
				Dir3::from(NeighborDirection::NegZ),
				Dir3::from(NeighborDirection::NegX),
				Dir3::from(NeighborDirection::Z),
				Dir3::from(NeighborDirection::X),
			]
		);
	}
}
