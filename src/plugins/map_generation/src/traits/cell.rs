use crate::{
	components::Corridor,
	map_loader::{Cell, Shape},
};
use bevy::math::primitives::Direction3d;
use common::traits::iteration::KeyValue;

impl KeyValue<Option<(Direction3d, String)>> for Cell {
	fn get_value(self) -> Option<(Direction3d, String)> {
		let value = match self {
			Cell::Corridor(direction, Shape::Single) => Some((direction, "single")),
			Cell::Corridor(direction, Shape::End) => Some((direction, "end")),
			Cell::Corridor(direction, Shape::Straight) => Some((direction, "straight")),
			Cell::Corridor(direction, Shape::Cross2) => Some((direction, "corner")),
			Cell::Corridor(direction, Shape::Cross3) => Some((direction, "t")),
			Cell::Corridor(direction, Shape::Cross4) => Some((direction, "cross")),
			Cell::Empty => None,
		};

		let (direction, suffix) = value?;

		Some((
			direction,
			format!("{}{}.glb#Scene0", Corridor::MODEL_PATH_PREFIX, suffix),
		))
	}
}
