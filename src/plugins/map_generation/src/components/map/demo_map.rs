use crate::components::map::{Map, cells::corridor::Corridor, folder::MapFolder};
use bevy::prelude::*;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[require(Map, MapFolder<Corridor> = MapFolder::from("maps/demo_map"))]
pub(crate) struct DemoMap;
