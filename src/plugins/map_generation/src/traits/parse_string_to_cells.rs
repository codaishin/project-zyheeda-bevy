use super::StringToCells;
use crate::{
	map_loader::{Cell, Cells, Shape},
	parsers::ParseStringToCells,
};
use bevy::math::primitives::Direction3d;

impl StringToCells for ParseStringToCells {
	fn string_to_cells(raw: &str) -> Cells {
		let lines: Vec<String> = raw
			.split('\n')
			.map(strip_white_spaces())
			.filter(non_empty())
			.collect();

		let cells = lines.iter().enumerate().map(parse(&lines)).collect();

		Cells(cells)
	}
}

fn parse(lines: &'_ [String]) -> impl FnMut((usize, &String)) -> Vec<Cell> + '_ {
	move |(line_i, line)| {
		line.chars()
			.enumerate()
			.map(|(char_i, char)| Cell::from(Cross::new(lines, line_i, char, char_i)))
			.collect()
	}
}

fn strip_white_spaces() -> impl FnMut(&str) -> String {
	|line| {
		line.chars()
			.filter(|c| !c.is_whitespace())
			.collect::<String>()
	}
}

fn non_empty() -> impl FnMut(&String) -> bool {
	|line| !line.is_empty()
}

struct Cross {
	middle: char,
	up: Option<char>,
	down: Option<char>,
	left: Option<char>,
	right: Option<char>,
}

impl Cross {
	fn new(lines: &[String], line_i: usize, char: char, char_i: usize) -> Self {
		Self {
			middle: char,
			up: line_i
				.checked_sub(1)
				.and_then(|line_i| lines[line_i].chars().nth(char_i)),
			down: line_i
				.checked_add(1)
				.filter(|line_i| line_i < &lines.len())
				.and_then(|line_i| lines[line_i].chars().nth(char_i)),
			left: char_i
				.checked_sub(1)
				.and_then(|char_i| lines[line_i].chars().nth(char_i)),
			right: char_i
				.checked_add(1)
				.and_then(|char_i| lines[line_i].chars().nth(char_i)),
		}
	}
}

impl From<Cross> for Cell {
	fn from(cross: Cross) -> Self {
		match cross {
			// Cross
			Cross {
				middle: 'c',
				up: Some('c'),
				down: Some('c'),
				right: Some('c'),
				left: Some('c'),
			} => Cell::Corridor(Direction3d::NEG_Z, Shape::Cross4),
			// T
			Cross {
				middle: 'c',
				up: Some('c'),
				down: Some('c'),
				left: Some('c'),
				..
			} => Cell::Corridor(Direction3d::NEG_X, Shape::Cross3),
			Cross {
				middle: 'c',
				up: Some('c'),
				left: Some('c'),
				right: Some('c'),
				..
			} => Cell::Corridor(Direction3d::Z, Shape::Cross3),
			Cross {
				middle: 'c',
				down: Some('c'),
				left: Some('c'),
				right: Some('c'),
				..
			} => Cell::Corridor(Direction3d::NEG_Z, Shape::Cross3),
			Cross {
				middle: 'c',
				up: Some('c'),
				down: Some('c'),
				right: Some('c'),
				..
			} => Cell::Corridor(Direction3d::X, Shape::Cross3),
			// Corners
			Cross {
				middle: 'c',
				up: Some('c'),
				left: Some('c'),
				..
			} => Cell::Corridor(Direction3d::Z, Shape::Cross2),
			Cross {
				middle: 'c',
				up: Some('c'),
				right: Some('c'),
				..
			} => Cell::Corridor(Direction3d::X, Shape::Cross2),
			Cross {
				middle: 'c',
				down: Some('c'),
				left: Some('c'),
				..
			} => Cell::Corridor(Direction3d::NEG_X, Shape::Cross2),
			Cross {
				middle: 'c',
				down: Some('c'),
				right: Some('c'),
				..
			} => Cell::Corridor(Direction3d::NEG_Z, Shape::Cross2),
			// Straights
			Cross {
				middle: 'c',
				right: Some('c'),
				left: Some('c'),
				..
			} => Cell::Corridor(Direction3d::X, Shape::Straight),
			Cross {
				middle: 'c',
				up: Some('c'),
				down: Some('c'),
				..
			} => Cell::Corridor(Direction3d::NEG_Z, Shape::Straight),
			// Ends
			Cross {
				middle: 'c',
				right: Some('c'),
				..
			} => Cell::Corridor(Direction3d::X, Shape::End),
			Cross {
				middle: 'c',
				left: Some('c'),
				..
			} => Cell::Corridor(Direction3d::NEG_X, Shape::End),
			Cross {
				middle: 'c',
				up: Some('c'),
				..
			} => Cell::Corridor(Direction3d::Z, Shape::End),
			Cross {
				middle: 'c',
				down: Some('c'),
				..
			} => Cell::Corridor(Direction3d::NEG_Z, Shape::End),
			// Single
			Cross { middle: 'c', .. } => Cell::Corridor(Direction3d::NEG_Z, Shape::Single),
			// None
			_ => Cell::Empty,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::map_loader::{Cell, Shape};

	#[test]
	fn single_empty() {
		let raw = "x";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(Cells(vec![vec![Cell::Empty]]), cells);
	}

	#[test]
	fn single_corridor() {
		let raw = "c";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(
			Cells(vec![vec![Cell::Corridor(
				Direction3d::NEG_Z,
				Shape::Single
			)]]),
			cells
		);
	}

	#[test]
	fn skip_white_spaces() {
		let raw = "c x";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(
			Cells(vec![vec![
				Cell::Corridor(Direction3d::NEG_Z, Shape::Single),
				Cell::Empty
			]]),
			cells
		);
	}

	#[test]
	fn process_multiple_lines() {
		let raw = "
		  cx
		  xc
		";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(
			Cells(vec![
				vec![
					Cell::Corridor(Direction3d::NEG_Z, Shape::Single),
					Cell::Empty
				],
				vec![
					Cell::Empty,
					Cell::Corridor(Direction3d::NEG_Z, Shape::Single)
				]
			]),
			cells
		);
	}

	#[test]
	fn corridor_ends() {
		let raw = "cc";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(
			Cells(vec![vec![
				Cell::Corridor(Direction3d::X, Shape::End),
				Cell::Corridor(Direction3d::NEG_X, Shape::End),
			]]),
			cells
		);
	}

	#[test]
	fn corridor_ends_with_straight_horizontally() {
		let raw = "ccc";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(
			Cells(vec![vec![
				Cell::Corridor(Direction3d::X, Shape::End),
				Cell::Corridor(Direction3d::X, Shape::Straight),
				Cell::Corridor(Direction3d::NEG_X, Shape::End),
			]]),
			cells
		);
	}

	#[test]
	fn corridor_ends_with_straight_vertically() {
		let raw = "
		  x c x
			x c x
      x c x
		";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(
			Cells(vec![
				vec![
					Cell::Empty,
					Cell::Corridor(Direction3d::NEG_Z, Shape::End),
					Cell::Empty,
				],
				vec![
					Cell::Empty,
					Cell::Corridor(Direction3d::NEG_Z, Shape::Straight),
					Cell::Empty,
				],
				vec![
					Cell::Empty,
					Cell::Corridor(Direction3d::Z, Shape::End),
					Cell::Empty,
				]
			]),
			cells
		);
	}

	#[test]
	fn corridor_ends_with_left_upper_corner() {
		let raw = "
		  c c
			c x
		";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(
			Cells(vec![
				vec![
					Cell::Corridor(Direction3d::NEG_Z, Shape::Cross2),
					Cell::Corridor(Direction3d::NEG_X, Shape::End),
				],
				vec![Cell::Corridor(Direction3d::Z, Shape::End), Cell::Empty,],
			]),
			cells
		);
	}

	#[test]
	fn corridor_ends_with_right_upper_corner() {
		let raw = "
		  c c
			x c
		";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(
			Cells(vec![
				vec![
					Cell::Corridor(Direction3d::X, Shape::End),
					Cell::Corridor(Direction3d::NEG_X, Shape::Cross2),
				],
				vec![Cell::Empty, Cell::Corridor(Direction3d::Z, Shape::End),],
			]),
			cells
		);
	}

	#[test]
	fn corridor_ends_with_left_lower_corner() {
		let raw = "
		  c x
		  c c
		";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(
			Cells(vec![
				vec![Cell::Corridor(Direction3d::NEG_Z, Shape::End), Cell::Empty,],
				vec![
					Cell::Corridor(Direction3d::X, Shape::Cross2),
					Cell::Corridor(Direction3d::NEG_X, Shape::End),
				],
			]),
			cells
		);
	}

	#[test]
	fn corridor_ends_with_right_lower_corner() {
		let raw = "
		  x c
		  c c
		";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(
			Cells(vec![
				vec![Cell::Empty, Cell::Corridor(Direction3d::NEG_Z, Shape::End),],
				vec![
					Cell::Corridor(Direction3d::X, Shape::End),
					Cell::Corridor(Direction3d::Z, Shape::Cross2),
				],
			]),
			cells
		);
	}

	#[test]
	fn corridor_ends_with_t_down() {
		let raw = "
		  c c c
		  x c x
		";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(
			Cells(vec![
				vec![
					Cell::Corridor(Direction3d::X, Shape::End),
					Cell::Corridor(Direction3d::NEG_Z, Shape::Cross3),
					Cell::Corridor(Direction3d::NEG_X, Shape::End),
				],
				vec![
					Cell::Empty,
					Cell::Corridor(Direction3d::Z, Shape::End),
					Cell::Empty,
				],
			]),
			cells
		);
	}

	#[test]
	fn corridor_ends_with_t_up() {
		let raw = "
			x c x
			c c c
		";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(
			Cells(vec![
				vec![
					Cell::Empty,
					Cell::Corridor(Direction3d::NEG_Z, Shape::End),
					Cell::Empty,
				],
				vec![
					Cell::Corridor(Direction3d::X, Shape::End),
					Cell::Corridor(Direction3d::Z, Shape::Cross3),
					Cell::Corridor(Direction3d::NEG_X, Shape::End),
				],
			]),
			cells
		);
	}

	#[test]
	fn corridor_ends_with_t_right() {
		let raw = "
			c x
			c c
			c x
		";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(
			Cells(vec![
				vec![Cell::Corridor(Direction3d::NEG_Z, Shape::End), Cell::Empty,],
				vec![
					Cell::Corridor(Direction3d::X, Shape::Cross3),
					Cell::Corridor(Direction3d::NEG_X, Shape::End),
				],
				vec![Cell::Corridor(Direction3d::Z, Shape::End), Cell::Empty,],
			]),
			cells
		);
	}

	#[test]
	fn corridor_ends_with_t_left() {
		let raw = "
			x c
			c c
			x c
		";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(
			Cells(vec![
				vec![Cell::Empty, Cell::Corridor(Direction3d::NEG_Z, Shape::End),],
				vec![
					Cell::Corridor(Direction3d::X, Shape::End),
					Cell::Corridor(Direction3d::NEG_X, Shape::Cross3),
				],
				vec![Cell::Empty, Cell::Corridor(Direction3d::Z, Shape::End),],
			]),
			cells
		);
	}

	#[test]
	fn corridor_ends_with_cross() {
		let raw = "
			x c x
			c c c
			x c x
		";
		let cells = ParseStringToCells::string_to_cells(raw);

		assert_eq!(
			Cells(vec![
				vec![
					Cell::Empty,
					Cell::Corridor(Direction3d::NEG_Z, Shape::End),
					Cell::Empty,
				],
				vec![
					Cell::Corridor(Direction3d::X, Shape::End),
					Cell::Corridor(Direction3d::NEG_Z, Shape::Cross4),
					Cell::Corridor(Direction3d::NEG_X, Shape::End),
				],
				vec![
					Cell::Empty,
					Cell::Corridor(Direction3d::Z, Shape::End),
					Cell::Empty,
				],
			]),
			cells
		);
	}
}
