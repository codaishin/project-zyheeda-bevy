use super::{map::Cross, CellDistance, SourcePath, Spawn};
use crate::{
	components::{Light, Point},
	map::LightCell,
};
use bevy::{
	ecs::system::Commands,
	math::primitives::Direction3d,
	transform::{components::Transform, TransformBundle},
};
use common::traits::load_asset::Path;

impl SourcePath for LightCell {
	fn source_path() -> Path {
		Path::from("maps/light_map.txt")
	}
}

impl From<LightCell> for Direction3d {
	fn from(value: LightCell) -> Direction3d {
		match value {
			LightCell::Point(direction) => direction,
			LightCell::Empty => Direction3d::NEG_Z,
		}
	}
}

impl CellDistance for LightCell {
	const CELL_DISTANCE: f32 = 2.;
}

impl Spawn for LightCell {
	fn spawn(&self, commands: &mut Commands, at: Transform) {
		let LightCell::Point(_) = self else {
			return;
		};
		commands.spawn((Light::<Point>::default(), TransformBundle::from(at)));
	}
}

impl From<Cross> for LightCell {
	fn from(cross: Cross) -> Self {
		match cross {
			Cross { middle: 'p', .. } => LightCell::Point(Direction3d::NEG_Z),
			_ => LightCell::Empty,
		}
	}
}
