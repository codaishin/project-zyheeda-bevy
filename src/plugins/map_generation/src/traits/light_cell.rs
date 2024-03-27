use super::{map::MapWindow, CellDistance, SourcePath, Spawn};
use crate::{
	components::{Floating, Light},
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
		Path::from("maps/map_floating_lights.txt")
	}
}

impl From<LightCell> for Direction3d {
	fn from(_: LightCell) -> Direction3d {
		Direction3d::NEG_Z
	}
}

impl CellDistance for LightCell {
	const CELL_DISTANCE: f32 = 2.;
}

impl Spawn for LightCell {
	fn spawn(&self, commands: &mut Commands, at: Transform) {
		let LightCell::Floating = self else {
			return;
		};
		commands.spawn((Light::<Floating>::default(), TransformBundle::from(at)));
	}
}

impl From<MapWindow> for LightCell {
	fn from(cross: MapWindow) -> Self {
		match cross {
			MapWindow { focus: 'f', .. } => LightCell::Floating,
			_ => LightCell::Empty,
		}
	}
}
