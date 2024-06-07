use super::AddAsset;
use bevy::asset::{Asset, Assets, Handle};

impl<TAsset: Asset> AddAsset<TAsset> for Assets<TAsset> {
	fn add_asset(&mut self, asset: TAsset) -> Handle<TAsset> {
		self.add(asset)
	}
}
