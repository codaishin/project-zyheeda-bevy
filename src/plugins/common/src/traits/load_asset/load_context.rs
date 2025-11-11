use super::LoadAsset;
use bevy::asset::{Asset, AssetPath, Handle, LoadContext};

impl LoadAsset for LoadContext<'_> {
	fn load_asset<'a, TAsset, TPath>(&mut self, path: TPath) -> Handle<TAsset>
	where
		TAsset: Asset,
		TPath: Into<AssetPath<'a>>,
	{
		self.load(path)
	}
}
