use super::LoadAsset;
use bevy::asset::{Asset, AssetPath, AssetServer, Handle};

impl<TAsset: Asset> LoadAsset<TAsset> for AssetServer {
	fn load_asset<'a, TPath: Into<AssetPath<'a>>>(&self, path: TPath) -> Handle<TAsset> {
		self.load(path)
	}
}
