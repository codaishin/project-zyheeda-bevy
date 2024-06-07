pub mod load_asset;

use bevy::asset::{Asset, Handle};

pub trait SharedAssetHandle<TCache, TKey, TAsset: Asset> {
	fn handle(&mut self, cache: &mut TCache, key: TKey) -> Handle<TAsset>;
}
