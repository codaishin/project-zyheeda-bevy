use super::Load;
use crate::tools::ModelPath;
use bevy::prelude::*;

impl Load<ModelPath, Handle<Scene>> for AssetServer {
	fn load(&self, ModelPath(path): &ModelPath) -> Handle<Scene> {
		self.load(GltfAssetLabel::Scene(0).from_asset(*path))
	}
}
