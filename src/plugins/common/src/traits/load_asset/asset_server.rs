use super::LoadAsset;
use bevy::{asset::AssetPath, prelude::*};

impl LoadAsset for AssetServer {
	fn load_asset<'a, TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
	where
		TAsset: Asset,
		TPath: Into<AssetPath<'a>>,
	{
		self.load(path)
	}
}
