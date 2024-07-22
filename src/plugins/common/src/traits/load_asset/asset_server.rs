use super::{LoadAsset, Path};
use bevy::asset::{Asset, AssetServer, Handle};

impl LoadAsset for AssetServer {
	fn load_asset<TAsset: Asset>(&mut self, path: Path) -> Handle<TAsset> {
		self.load(path)
	}
}
