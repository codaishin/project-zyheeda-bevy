pub(crate) mod corridor;
pub(crate) mod half_offset_cell;

use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;
use half_offset_cell::HalfOffsetCell;
use std::fmt::Display;

#[derive(TypePath, Asset, Debug, PartialEq)]
pub(crate) struct MapCells<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	cells: Vec<Vec<TCell>>,
	half_offset_cells: Vec<Vec<HalfOffsetCell<TCell>>>,
}

impl<TCell> MapCells<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	pub(crate) fn cells(&self) -> &Vec<Vec<TCell>> {
		&self.cells
	}

	pub(crate) fn half_offset_cells(&self) -> &Vec<Vec<HalfOffsetCell<TCell>>> {
		&self.half_offset_cells
	}

	#[cfg(test)]
	pub(crate) fn new(cells: Vec<Vec<TCell>>, quadrants: Vec<Vec<HalfOffsetCell<TCell>>>) -> Self {
		Self {
			cells,
			half_offset_cells: quadrants,
		}
	}
}

impl<TCell> Default for MapCells<TCell>
where
	TCell: TypePath + ThreadSafe,
{
	fn default() -> Self {
		Self {
			cells: vec![],
			half_offset_cells: vec![],
		}
	}
}

impl<TCell> TryFrom<String> for MapCells<TCell>
where
	TCell: From<Option<char>> + TypePath + Clone + ThreadSafe,
{
	type Error = MapSizeError;

	fn try_from(value: String) -> Result<Self, Self::Error> {
		let lines: Vec<String> = value
			.split('\n')
			.map(strip_white_spaces)
			.filter(|line| non_empty(line))
			.collect();

		let mut max_line_length = 0;
		let mut cells = lines
			.iter()
			.map(parse_cells::<TCell>(&mut max_line_length))
			.collect::<Vec<_>>();

		fill_lines(&mut cells, max_line_length);

		if let Some(error) = MapSizeError::check(&cells) {
			return Err(error);
		}

		let half_offset_cells = cells
			.iter()
			.enumerate()
			.filter(greater_than(0))
			.map(half_offset_cells(&cells))
			.collect();

		Ok(Self {
			cells,
			half_offset_cells,
		})
	}
}

fn half_offset_cells<TCell>(
	grid: &[Vec<TCell>],
) -> impl FnMut((usize, &Vec<TCell>)) -> Vec<HalfOffsetCell<TCell>>
where
	TCell: Clone,
{
	|(z, line)| {
		line.iter()
			.enumerate()
			.filter(greater_than(0))
			.map(|(x, _)| HalfOffsetCell::from(half_offset_quadrants(z, x, grid)))
			.collect()
	}
}

fn half_offset_quadrants<TCell>(z: usize, x: usize, grid: &[Vec<TCell>]) -> [(Direction, TCell); 4]
where
	TCell: Clone,
{
	HalfOffsetCell::with_quadrants(x, z).map(|(x, z, dir)| (dir, grid[z][x].clone()))
}

fn greater_than<T>(than: usize) -> impl FnMut(&(usize, &T)) -> bool {
	move |(v, ..)| v > &than
}

fn fill_lines<TCell>(cells: &mut Vec<Vec<TCell>>, max_line_length: usize)
where
	TCell: From<Option<char>> + TypePath + ThreadSafe,
{
	for cells in cells {
		while cells.len() < max_line_length {
			cells.push(TCell::from(None));
		}
	}
}

fn parse_cells<TCell>(max_x: &mut usize) -> impl FnMut(&String) -> Vec<TCell>
where
	TCell: From<Option<char>> + TypePath + ThreadSafe,
{
	|line| {
		let line = line
			.chars()
			.map(|c| TCell::from(Some(c)))
			.collect::<Vec<_>>();
		*max_x = usize::max(line.len(), *max_x);
		line
	}
}

#[derive(Debug, PartialEq)]
pub(crate) enum MapSizeError {
	Empty,
	Sizes { x: usize, z: usize },
}

impl MapSizeError {
	fn check<TCell>(cells: &Vec<Vec<TCell>>) -> Option<Self> {
		match cells.as_slice() {
			[] => Some(MapSizeError::Empty),
			[fst] => Some(MapSizeError::Sizes {
				x: fst.len(),
				z: cells.len(),
			}),
			[fst, _, ..] if fst.len() < 2 => Some(MapSizeError::Sizes {
				x: fst.len(),
				z: cells.len(),
			}),
			_ => None,
		}
	}
}

impl Display for MapSizeError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"Minimum map size of 2x2 required, but map was {:?}",
			self
		)
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

#[cfg(test)]
mod tests {
	use super::*;
	use test_case::test_case;

	#[derive(TypePath, Debug, PartialEq, Clone)]
	struct _Cell(Option<char>);

	impl From<Option<char>> for _Cell {
		fn from(value: Option<char>) -> Self {
			_Cell(value)
		}
	}

	#[test]
	fn empty() {
		let raw = "".to_string();
		let map = MapCells::<_Cell>::try_from(raw);

		assert_eq!(Err(MapSizeError::Empty), map);
	}

	#[test]
	fn too_small_z() {
		let raw = "cx".to_string();
		let map = MapCells::<_Cell>::try_from(raw);

		assert_eq!(Err(MapSizeError::Sizes { x: 2, z: 1 }), map);
	}

	#[test]
	fn too_small_x_when_z_2() {
		let raw = "
		  c
			t
		"
		.to_string();
		let map = MapCells::<_Cell>::try_from(raw);

		assert_eq!(Err(MapSizeError::Sizes { x: 1, z: 2 }), map);
	}

	#[test]
	fn too_small_x_when_z_3() {
		let raw = "
		  c
			t
			u
		"
		.to_string();
		let map = MapCells::<_Cell>::try_from(raw);

		assert_eq!(Err(MapSizeError::Sizes { x: 1, z: 3 }), map);
	}

	#[test]
	fn parse_character() -> Result<(), MapSizeError> {
		let raw = "xc\nty".to_string();
		let map = MapCells::<_Cell>::try_from(raw)?;

		assert_eq!(
			MapCells {
				cells: vec![
					vec![_Cell(Some('x')), _Cell(Some('c'))],
					vec![_Cell(Some('t')), _Cell(Some('y'))]
				],
				half_offset_cells: vec![vec![HalfOffsetCell::from([
					(Direction::Z, _Cell(Some('x'))),
					(Direction::X, _Cell(Some('t'))),
					(Direction::NegZ, _Cell(Some('y'))),
					(Direction::NegX, _Cell(Some('c'))),
				])]],
			},
			map
		);
		Ok(())
	}

	#[test]
	fn skip_white_spaces() -> Result<(), MapSizeError> {
		let raw = "
	    x c
		  t y
	  "
		.to_string();
		let map = MapCells::<_Cell>::try_from(raw)?;

		assert_eq!(
			MapCells {
				cells: vec![
					vec![_Cell(Some('x')), _Cell(Some('c'))],
					vec![_Cell(Some('t')), _Cell(Some('y'))]
				],
				half_offset_cells: vec![vec![HalfOffsetCell::from([
					(Direction::Z, _Cell(Some('x'))),
					(Direction::X, _Cell(Some('t'))),
					(Direction::NegZ, _Cell(Some('y'))),
					(Direction::NegX, _Cell(Some('c'))),
				])]],
			},
			map
		);
		Ok(())
	}

	#[test]
	fn process_multiple_lines() -> Result<(), MapSizeError> {
		let raw = "
			xct
			erc
		"
		.to_string();
		let map = MapCells::try_from(raw)?;

		assert_eq!(
			MapCells {
				cells: vec![
					vec![_Cell(Some('x')), _Cell(Some('c')), _Cell(Some('t'))],
					vec![_Cell(Some('e')), _Cell(Some('r')), _Cell(Some('c'))]
				],
				half_offset_cells: vec![vec![
					HalfOffsetCell::from([
						(Direction::Z, _Cell(Some('x'))),
						(Direction::X, _Cell(Some('e'))),
						(Direction::NegZ, _Cell(Some('r'))),
						(Direction::NegX, _Cell(Some('c'))),
					]),
					HalfOffsetCell::from([
						(Direction::Z, _Cell(Some('c'))),
						(Direction::X, _Cell(Some('r'))),
						(Direction::NegZ, _Cell(Some('c'))),
						(Direction::NegX, _Cell(Some('t'))),
					])
				]],
			},
			map
		);
		Ok(())
	}

	#[test]
	fn process_multiple_lines_with_uneven_lengths() -> Result<(), MapSizeError> {
		let raw = "
			xct
			frog
			er
		"
		.to_string();
		let map = MapCells::try_from(raw)?;

		assert_eq!(
			MapCells {
				cells: vec![
					vec![
						_Cell(Some('x')),
						_Cell(Some('c')),
						_Cell(Some('t')),
						_Cell(None)
					],
					vec![
						_Cell(Some('f')),
						_Cell(Some('r')),
						_Cell(Some('o'),),
						_Cell(Some('g'))
					],
					vec![_Cell(Some('e')), _Cell(Some('r')), _Cell(None), _Cell(None)],
				],
				half_offset_cells: vec![
					vec![
						HalfOffsetCell::from([
							(Direction::Z, _Cell(Some('x'))),
							(Direction::X, _Cell(Some('f'))),
							(Direction::NegZ, _Cell(Some('r'))),
							(Direction::NegX, _Cell(Some('c'))),
						]),
						HalfOffsetCell::from([
							(Direction::Z, _Cell(Some('c'))),
							(Direction::X, _Cell(Some('r'))),
							(Direction::NegZ, _Cell(Some('o'))),
							(Direction::NegX, _Cell(Some('t'))),
						]),
						HalfOffsetCell::from([
							(Direction::Z, _Cell(Some('t'))),
							(Direction::X, _Cell(Some('o'))),
							(Direction::NegZ, _Cell(Some('g'))),
							(Direction::NegX, _Cell(None)),
						]),
					],
					vec![
						HalfOffsetCell::from([
							(Direction::Z, _Cell(Some('f'))),
							(Direction::X, _Cell(Some('e'))),
							(Direction::NegZ, _Cell(Some('r'))),
							(Direction::NegX, _Cell(Some('r'))),
						]),
						HalfOffsetCell::from([
							(Direction::Z, _Cell(Some('r'))),
							(Direction::X, _Cell(Some('r'))),
							(Direction::NegZ, _Cell(None)),
							(Direction::NegX, _Cell(Some('o'))),
						]),
						HalfOffsetCell::from([
							(Direction::Z, _Cell(Some('o'))),
							(Direction::X, _Cell(None)),
							(Direction::NegZ, _Cell(None)),
							(Direction::NegX, _Cell(Some('g'))),
						]),
					]
				],
			},
			map
		);
		Ok(())
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
