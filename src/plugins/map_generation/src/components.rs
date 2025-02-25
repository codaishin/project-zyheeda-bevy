pub(crate) mod level;

use crate::map::Map;
use bevy::prelude::*;

pub(crate) struct Wall;

pub(crate) struct WallBack;

pub(crate) struct Corridor;

impl Corridor {
	pub const MODEL_PATH_PREFIX: &'static str = "models/corridor_";
}

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct LoadLevelCommand<TCell: TypePath + Send + Sync>(pub Handle<Map<TCell>>);

#[derive(Component)]
pub(crate) struct Unlit;
