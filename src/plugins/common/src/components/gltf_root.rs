use crate::components::asset_model::SceneId;
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct GltfRoot {
	pub(crate) gltf: Handle<Gltf>,
	pub(crate) id: SceneId,
}
