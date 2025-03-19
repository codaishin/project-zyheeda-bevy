pub(crate) mod grid_min;
pub(crate) mod insert_cell_components;
pub(crate) mod is_walkable;
pub(crate) mod key_mapper;
pub(crate) mod light;
pub(crate) mod load_map;
pub(crate) mod to_subdivided;
pub(crate) mod wall;

use bevy::prelude::*;
use common::traits::{handles_lights::HandlesLights, load_asset::Path, thread_safe::ThreadSafe};

pub(crate) trait ExtraComponentsDefinition {
	fn target_names() -> Vec<String>;
	fn insert_bundle<TLights>(entity: &mut EntityCommands)
	where
		TLights: HandlesLights + ThreadSafe;
}

pub(crate) trait GridCellDistanceDefinition {
	const CELL_DISTANCE: f32;
}

pub trait SourcePath {
	fn source_path() -> Path;
}
