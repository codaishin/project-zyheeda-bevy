use crate::map::Map;
use bevy::reflect::TypePath;

impl<TCell: From<Cross> + TypePath + Sync + Send> From<String> for Map<TCell> {
	fn from(value: String) -> Self {
		let lines: Vec<String> = value
			.split('\n')
			.map(strip_white_spaces)
			.filter(|line| non_empty(line))
			.collect();

		let map = lines.iter().enumerate().map(parse(&lines)).collect();

		Self(map)
	}
}

fn parse<TCell: From<Cross>>(
	lines: &'_ [String],
) -> impl FnMut((usize, &String)) -> Vec<TCell> + '_ {
	move |(line_i, line)| {
		line.chars()
			.enumerate()
			.map(|(char_i, char)| TCell::from(Cross::new(lines, line_i, char, char_i)))
			.collect()
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

#[derive(Default, Debug, PartialEq)]
pub(crate) struct Cross {
	pub middle: char,
	pub up: Option<char>,
	pub down: Option<char>,
	pub left: Option<char>,
	pub right: Option<char>,
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

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;

	#[derive(TypePath, Debug, PartialEq)]
	struct _Cell(Cross);

	impl From<Cross> for _Cell {
		fn from(value: Cross) -> Self {
			_Cell(value)
		}
	}

	#[test]
	fn single() {
		let raw = "x".to_string();
		let map = Map::<_Cell>::from(raw);

		assert_eq!(
			Map(vec![vec![_Cell(Cross {
				middle: 'x',
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
				_Cell(Cross {
					middle: 'c',
					right: Some('x'),
					..default()
				}),
				_Cell(Cross {
					middle: 'x',
					left: Some('c'),
					..default()
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
				_Cell(Cross {
					middle: 'x',
					right: Some('c'),
					..default()
				}),
				_Cell(Cross {
					middle: 'c',
					left: Some('x'),
					..default()
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
					_Cell(Cross {
						middle: 'x',
						down: Some('e'),
						right: Some('c'),
						..default()
					}),
					_Cell(Cross {
						middle: 'c',
						down: Some('r'),
						left: Some('x'),
						right: Some('t'),
						..default()
					}),
					_Cell(Cross {
						middle: 't',
						down: Some('j'),
						left: Some('c'),
						..default()
					}),
				],
				vec![
					_Cell(Cross {
						middle: 'e',
						up: Some('x'),
						down: Some('l'),
						right: Some('r'),
						..default()
					}),
					_Cell(Cross {
						middle: 'r',
						up: Some('c'),
						down: Some('p'),
						left: Some('e'),
						right: Some('j'),
					}),
					_Cell(Cross {
						middle: 'j',
						up: Some('t'),
						down: Some('n'),
						left: Some('r'),
						..default()
					}),
				],
				vec![
					_Cell(Cross {
						middle: 'l',
						up: Some('e'),
						right: Some('p'),
						..default()
					}),
					_Cell(Cross {
						middle: 'p',
						up: Some('r'),
						right: Some('n'),
						left: Some('l'),
						..default()
					}),
					_Cell(Cross {
						middle: 'n',
						up: Some('j'),
						left: Some('p'),
						..default()
					}),
				]
			]),
			map
		);
	}
}
