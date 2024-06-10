pub mod get_or_create_asset;
pub mod get_or_load_asset;

use super::load_asset::Path;
use bevy::asset::{Asset, Handle};

pub trait Cache<TKey, TValue> {
	fn cached(&mut self, key: TKey, new: impl FnOnce() -> TValue) -> TValue;
}

pub trait GetOrLoadAsset<TAsset: Asset> {
	fn get_or_load(&mut self, key: Path) -> Handle<TAsset>;
}

pub trait GetOrCreateAsset<TKey, TAsset: Asset> {
	fn get_or_create(&mut self, key: TKey, create: impl FnOnce() -> TAsset) -> Handle<TAsset>;
}
