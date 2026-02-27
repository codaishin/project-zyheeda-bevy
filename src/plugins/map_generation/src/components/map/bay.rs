use crate::components::map::Map;
use bevy::prelude::*;
use common::components::asset_model::AssetModel;
use macros::SavableComponent;
use serde::{Deserialize, Serialize};

#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone, Serialize, Deserialize)]
#[savable_component(id = "bay map")]
#[require(Map, Name = "BayMap", AssetModel = AssetModel::path("maps/bay.glb"))]
pub(crate) struct BayMap;
