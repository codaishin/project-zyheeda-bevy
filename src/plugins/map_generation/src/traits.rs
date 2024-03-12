pub(crate) mod asset_server;
pub(crate) mod corner;
pub(crate) mod wall;

use crate::map_loader::Map;
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
