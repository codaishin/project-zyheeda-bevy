use crate::components::map::{asset::MapAsset, cells::corridor::Corridor};
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Default)]
#[require(MapAsset<Corridor> = MapAsset::from("maps/map.png"))]
pub(crate) struct DemoMap;
