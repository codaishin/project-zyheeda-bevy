pub(crate) mod cell;

use bevy::{asset::Asset, reflect::TypePath};

#[derive(TypePath, Asset, Debug, PartialEq)]
pub(crate) struct Map<TCell: TypePath + Sync + Send>(pub Vec<Vec<TCell>>);
