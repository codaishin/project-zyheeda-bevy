use super::{LoadAsset, Path};
use bevy::asset::{Asset, AssetServer, Handle};

impl<TAsset: Asset> LoadAsset<TAsset> for AssetServer {
	fn load_asset(&mut self, path: Path) -> Handle<TAsset> {
		self.load(path)
	}
}
