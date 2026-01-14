use crate::components::{asset_model::AssetModel, insert_asset::InsertAsset};
use bevy::prelude::*;

/// A common model definition component
///
/// The specific model will be inserted via observers in the [`CommonPlugin`](crate::CommonPlugin)
#[derive(Component, Debug, PartialEq, Clone)]
pub enum Model {
	Asset(AssetModel),
	Procedural(InsertAsset<Mesh>),
}
