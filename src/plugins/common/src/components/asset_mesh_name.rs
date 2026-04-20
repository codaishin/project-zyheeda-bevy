use bevy::prelude::*;
use zyheeda_core::prelude::NormalizedName;

#[derive(Component, Debug, PartialEq)]
pub struct AssetMeshName(pub NormalizedName<String>);
