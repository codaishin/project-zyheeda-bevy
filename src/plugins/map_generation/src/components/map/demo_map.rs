use crate::{components::map::MapAssetPath, map_cells::corridor::Corridor};
use bevy::prelude::*;
use common::traits::load_asset::Path;

#[derive(Component, Debug, PartialEq, Default)]
#[require(MapAssetPath<Corridor> = MapAssetPath::from(Path::from("maps/map.txt")))]
pub(crate) struct DemoMap;
