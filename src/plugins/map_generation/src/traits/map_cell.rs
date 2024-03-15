use super::{map::Cross, CellDistance};
use crate::{
	components::Corridor,
	map::{MapCell, Shape},
};
use bevy::math::primitives::Direction3d;
use common::traits::load_asset::Path;

pub(crate) struct CellIsEmpty;

impl TryFrom<MapCell> for Path {
	type Error = CellIsEmpty;

	fn try_from(value: MapCell) -> Result<Self, Self::Error> {
		let name = match value {
			MapCell::Corridor(_, Shape::Single) => Ok("single"),
			MapCell::Corridor(_, Shape::End) => Ok("end"),
			MapCell::Corridor(_, Shape::Straight) => Ok("straight"),
			MapCell::Corridor(_, Shape::Cross2) => Ok("corner"),
			MapCell::Corridor(_, Shape::Cross3) => Ok("t"),
			MapCell::Corridor(_, Shape::Cross4) => Ok("cross"),
			MapCell::Empty => Err(CellIsEmpty),
		};

		Ok(Path::from(format!(
			"{}{}.glb#Scene0",
			Corridor::MODEL_PATH_PREFIX,
			name?
		)))
	}
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

impl From<Cross> for MapCell {
	fn from(cross: Cross) -> Self {
		match cross {
			// Cross
			Cross {
				middle: 'c',
				up: Some('c'),
				down: Some('c'),
				right: Some('c'),
				left: Some('c'),
			} => MapCell::Corridor(Direction3d::NEG_Z, Shape::Cross4),
			// T
			Cross {
				middle: 'c',
				up: Some('c'),
				down: Some('c'),
				left: Some('c'),
				..
			} => MapCell::Corridor(Direction3d::X, Shape::Cross3),
			Cross {
				middle: 'c',
				up: Some('c'),
				left: Some('c'),
				right: Some('c'),
				..
			} => MapCell::Corridor(Direction3d::Z, Shape::Cross3),
			Cross {
				middle: 'c',
				down: Some('c'),
				left: Some('c'),
				right: Some('c'),
				..
			} => MapCell::Corridor(Direction3d::NEG_Z, Shape::Cross3),
			Cross {
				middle: 'c',
				up: Some('c'),
				down: Some('c'),
				right: Some('c'),
				..
			} => MapCell::Corridor(Direction3d::NEG_X, Shape::Cross3),
			// Corners
			Cross {
				middle: 'c',
				up: Some('c'),
				left: Some('c'),
				..
			} => MapCell::Corridor(Direction3d::X, Shape::Cross2),
			Cross {
				middle: 'c',
				up: Some('c'),
				right: Some('c'),
				..
			} => MapCell::Corridor(Direction3d::Z, Shape::Cross2),
			Cross {
				middle: 'c',
				down: Some('c'),
				left: Some('c'),
				..
			} => MapCell::Corridor(Direction3d::NEG_Z, Shape::Cross2),
			Cross {
				middle: 'c',
				down: Some('c'),
				right: Some('c'),
				..
			} => MapCell::Corridor(Direction3d::NEG_X, Shape::Cross2),
			// Straights
			Cross {
				middle: 'c',
				right: Some('c'),
				left: Some('c'),
				..
			} => MapCell::Corridor(Direction3d::NEG_X, Shape::Straight),
			Cross {
				middle: 'c',
				up: Some('c'),
				down: Some('c'),
				..
			} => MapCell::Corridor(Direction3d::NEG_Z, Shape::Straight),
			// Ends
			Cross {
				middle: 'c',
				right: Some('c'),
				..
			} => MapCell::Corridor(Direction3d::NEG_X, Shape::End),
			Cross {
				middle: 'c',
				left: Some('c'),
				..
			} => MapCell::Corridor(Direction3d::X, Shape::End),
			Cross {
				middle: 'c',
				up: Some('c'),
				..
			} => MapCell::Corridor(Direction3d::Z, Shape::End),
			Cross {
				middle: 'c',
				down: Some('c'),
				..
			} => MapCell::Corridor(Direction3d::NEG_Z, Shape::End),
			// Single
			Cross { middle: 'c', .. } => MapCell::Corridor(Direction3d::NEG_Z, Shape::Single),
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
		let cross = Cross {
			middle: 'x',
			..default()
		};

		assert_eq!(MapCell::Empty, MapCell::from(cross));
	}

	#[test]
	fn corridor_end_right() {
		let cross = Cross {
			middle: 'c',
			right: Some('c'),
			..default()
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_X, Shape::End),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_end_left() {
		let cross = Cross {
			middle: 'c',
			left: Some('c'),
			..default()
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::X, Shape::End),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_straight_horizontally() {
		let cross = Cross {
			middle: 'c',
			left: Some('c'),
			right: Some('c'),
			..default()
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_X, Shape::Straight),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_straight_vertically() {
		let cross = Cross {
			middle: 'c',
			left: Some('c'),
			right: Some('c'),
			..default()
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_X, Shape::Straight),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_left_upper_corner() {
		let cross = Cross {
			middle: 'c',
			down: Some('c'),
			right: Some('c'),
			..default()
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_X, Shape::Cross2),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_right_upper_corner() {
		let cross = Cross {
			middle: 'c',
			down: Some('c'),
			left: Some('c'),
			..default()
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_Z, Shape::Cross2),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_left_lower_corner() {
		let cross = Cross {
			middle: 'c',
			right: Some('c'),
			up: Some('c'),
			..default()
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::Z, Shape::Cross2),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_right_lower_corner() {
		let cross = Cross {
			middle: 'c',
			left: Some('c'),
			up: Some('c'),
			..default()
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::X, Shape::Cross2),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_t_down() {
		let cross = Cross {
			middle: 'c',
			left: Some('c'),
			right: Some('c'),
			down: Some('c'),
			..default()
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_Z, Shape::Cross3),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_t_up() {
		let cross = Cross {
			middle: 'c',
			left: Some('c'),
			right: Some('c'),
			up: Some('c'),
			..default()
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::Z, Shape::Cross3),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_t_right() {
		let cross = Cross {
			middle: 'c',
			down: Some('c'),
			right: Some('c'),
			up: Some('c'),
			..default()
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_X, Shape::Cross3),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_t_left() {
		let cross = Cross {
			middle: 'c',
			down: Some('c'),
			left: Some('c'),
			up: Some('c'),
			..default()
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::X, Shape::Cross3),
			MapCell::from(cross)
		);
	}

	#[test]
	fn corridor_cross() {
		let cross = Cross {
			middle: 'c',
			up: Some('c'),
			down: Some('c'),
			left: Some('c'),
			right: Some('c'),
		};

		assert_eq!(
			MapCell::Corridor(Direction3d::NEG_Z, Shape::Cross4),
			MapCell::from(cross)
		);
	}
}
