pub(crate) mod grid_min;
pub(crate) mod insert_cell_components;
pub(crate) mod insert_cell_quadrant_components;
pub(crate) mod is_walkable;
pub(crate) mod key_mapper;
pub(crate) mod parse_map_image;
pub(crate) mod pixels;
pub(crate) mod register_map_cell;
pub(crate) mod to_subdivided;

use crate::grid_graph::grid_context::CellDistance;
use bevy::prelude::*;
use common::traits::{handles_lights::HandlesLights, thread_safe::ThreadSafe};

pub(crate) trait ExtraComponentsDefinition {
	fn target_names() -> Vec<String>;
	fn insert_bundle<TLights>(entity: &mut EntityCommands)
	where
		TLights: HandlesLights + ThreadSafe;
}

pub(crate) trait GridCellDistanceDefinition {
	const CELL_DISTANCE: CellDistance;
}
