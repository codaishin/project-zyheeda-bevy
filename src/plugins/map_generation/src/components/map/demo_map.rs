use crate::{components::map::Map, map_cells::corridor::Corridor};
use bevy::prelude::*;
use common::traits::load_asset::Path;

#[derive(Component, Debug, PartialEq, Default)]
#[require(Map<Corridor> = Map::from_asset(Path::from("maps/map.txt")))]
pub(crate) struct DemoMap;
