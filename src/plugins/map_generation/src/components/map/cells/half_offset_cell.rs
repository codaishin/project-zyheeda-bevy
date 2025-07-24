use crate::components::map::cells::Direction;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub(crate) struct HalfOffsetCell<TCell> {
	quadrants: HashMap<Direction, TCell>,
}

impl HalfOffsetCell<()> {
	pub(crate) fn directions(x: u32, z: u32) -> [(u32, u32, Direction); 4] {
		[
			(x - 1, z - 1, Direction::Z),
			(x - 1, z, Direction::X),
			(x, z - 1, Direction::NegX),
			(x, z, Direction::NegZ),
		]
	}
}

impl<TCell> HalfOffsetCell<TCell> {
	pub(crate) fn quadrants(&self) -> &HashMap<Direction, TCell> {
		&self.quadrants
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
