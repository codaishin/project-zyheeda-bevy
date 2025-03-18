use super::Direction;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub(crate) struct HalfOffsetCell<TCell> {
	quadrants: HashMap<Direction, TCell>,
}

impl HalfOffsetCell<()> {
	pub(crate) fn quadrants(x: usize, z: usize) -> [(usize, usize, Direction); 4] {
		[
			(x - 1, z - 1, Direction::Z),
			(x - 1, z, Direction::NegX),
			(x, z, Direction::NegZ),
			(x, z - 1, Direction::X),
		]
	}
}

impl<TCell> Default for HalfOffsetCell<TCell> {
	fn default() -> Self {
		Self {
			quadrants: HashMap::default(),
		}
	}
}

impl<TCell, const N: usize> From<[(Direction, TCell); N]> for HalfOffsetCell<TCell> {
	fn from(quadrants: [(Direction, TCell); N]) -> Self {
		Self {
			quadrants: HashMap::from(quadrants),
		}
	}
}
