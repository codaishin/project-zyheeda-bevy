pub mod get_or_create_asset;
pub mod get_or_create_type_asset;
pub mod get_or_load_asset;

use super::load_asset::Path;
use bevy::{
	asset::{Asset, Handle},
	prelude::{ResMut, Resource},
};

pub trait Storage<TKey, T> {
	fn get_or_create(&mut self, key: TKey, create: impl FnMut() -> T) -> T;
}

pub trait GetOrLoadAsset<T: Asset> {
	fn get_or_load(&mut self, key: Path) -> Handle<T>;
}

pub trait GetOrCreateAsset<TKey, T: Asset> {
	fn get_or_create(&mut self, key: TKey, create: impl FnMut() -> T) -> Handle<T>;
}

pub trait GetOrCreateTypeAsset<T: Asset> {
	fn get_or_create_for<Key: 'static>(&mut self, create: impl FnMut() -> T) -> Handle<T>;
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
