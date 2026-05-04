use crate::components::asset_model::SceneId;
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(GltfScene)]
pub struct GltfLookup(pub Handle<Gltf>);

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct GltfScene(pub(crate) SceneId);
