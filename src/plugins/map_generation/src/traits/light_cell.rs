use super::{
	GridCellDistanceDefinition,
	SourcePath,
	Spawn,
	light::floating::FloatingLight,
	map::MapWindow,
};
use crate::map::LightCell;
use bevy::prelude::*;
use common::traits::load_asset::Path;

impl SourcePath for LightCell {
	fn source_path() -> Path {
		Path::from("maps/map_floating_lights.txt")
	}
}

impl From<&LightCell> for Dir3 {
	fn from(_: &LightCell) -> Dir3 {
		Dir3::NEG_Z
	}
}

impl GridCellDistanceDefinition for LightCell {
	const CELL_DISTANCE: f32 = 2.;
}

impl Spawn for LightCell {
	fn spawn(&self, commands: &mut Commands, at: Transform) {
		let LightCell::Floating = self else {
			return;
		};
		commands.spawn((FloatingLight, at));
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
