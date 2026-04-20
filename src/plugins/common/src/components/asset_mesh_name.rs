use bevy::prelude::*;
use zyheeda_core::prelude::NormalizedName;

/// Represents a normalized asset mesh name.
///
/// It is automatically inserted by the [`CommonPlugin`](crate::CommonPlugin) when
/// [`GltfMeshName`](bevy::gltf::GltfMeshName) is inserted .
#[derive(Component, Debug, PartialEq)]
#[component(immutable)]
pub struct AssetMeshName(pub NormalizedName<String>);
