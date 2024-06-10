pub mod get_or_create_asset;
pub mod get_or_create_type_asset;
pub mod get_or_load_asset;

use super::load_asset::Path;
use bevy::{
	asset::{Asset, Handle},
	prelude::{ResMut, Resource},
};

pub trait Cache<TKey, TValue> {
	fn cached(&mut self, key: TKey, new: impl FnOnce() -> TValue) -> TValue;
}

pub trait GetOrLoadAsset<TAsset: Asset> {
	fn get_or_load(&mut self, key: Path) -> Handle<TAsset>;
}

pub trait GetOrCreateAsset<TKey, TAsset: Asset> {
	fn get_or_create(&mut self, key: TKey, create: impl FnOnce() -> TAsset) -> Handle<TAsset>;
}

pub trait GetOrCreateTypeAsset<TAsset: Asset> {
	fn get_or_create<Key: 'static>(&mut self, create: impl FnOnce() -> TAsset) -> Handle<TAsset>;
}

pub trait GetOrLoadAssetFactory<TAssets, TAsset: Asset, TStorage>
where
	TAssets: Resource,
	TStorage: Resource,
{
	fn create_from(
		assets: ResMut<TAssets>,
		storage: ResMut<TStorage>,
	) -> impl GetOrLoadAsset<TAsset>;
}

pub trait GetOrCreateAssetFactory<TAssets, TAsset: Asset, TStorage, TKey>
where
	TAssets: Resource,
	TStorage: Resource,
{
	fn create_from(
		assets: ResMut<TAssets>,
		storage: ResMut<TStorage>,
	) -> impl GetOrCreateAsset<TKey, TAsset>;
}
