use crate::components::map::{cells::corridor::Corridor, folder::MapFolder};
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq, Default)]
#[require(MapFolder<Corridor> = MapFolder::from("maps/demo_map"))]
pub(crate) struct DemoMap;
