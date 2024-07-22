use super::LoadAsset;
use bevy::asset::{Asset, Handle, LoadContext};

impl<'a> LoadAsset for LoadContext<'a> {
	fn load_asset<TAsset: Asset>(&mut self, path: super::Path) -> Handle<TAsset> {
		self.load(path)
	}
}
