pub(crate) mod asset_server;
pub(crate) mod cell;
pub(crate) mod corner;
pub(crate) mod map;
pub(crate) mod wall;

use crate::map::Map;
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

pub(crate) trait MapAsset<TCell> {
	const CELL_DISTANCE: f32;
	fn cells(&self) -> Vec<Vec<TCell>>;
}
