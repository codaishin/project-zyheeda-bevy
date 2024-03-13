pub(crate) mod asset_server;
pub(crate) mod corner;
pub(crate) mod parse_string_to_cells;
pub(crate) mod wall;

use crate::map_loader::{Cells, Map};
use bevy::asset::Handle;
use bevy_rapier3d::geometry::Collider;

pub(crate) trait ColliderDefinition {
	const IS_TARGET: bool;
	fn target_names() -> Vec<String>;
	fn collider() -> Collider;
}

pub(crate) trait LoadMap {
	fn load(&self) -> Handle<Map>;
}

pub(crate) trait StringToCells {
	fn string_to_cells(raw: &str) -> Cells;
}
