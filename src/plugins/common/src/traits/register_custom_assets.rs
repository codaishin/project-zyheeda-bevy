use super::load_asset::{LoadAsset, Path};
use bevy::prelude::*;
use serde::Deserialize;
use std::fmt::Debug;

pub trait RegisterCustomAssets {
	fn register_custom_assets<TAsset, TDto>(app: &mut App)
	where
		TAsset: Asset + LoadFrom<TDto> + Clone + Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + Sync + Send + 'static;
}

pub trait RegisterCustomFolderAssets {
	fn register_custom_folder_assets<TAsset, TDto>(app: &mut App)
	where
		TAsset: Asset + AssetFolderPath + LoadFrom<TDto> + Clone + Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + Sync + Send + 'static;
}

pub trait AssetFolderPath {
	fn asset_folder_path() -> Path;
}

pub trait LoadFrom<TFrom> {
	fn load_from<TLoadAsset: LoadAsset>(from: TFrom, asset_server: &mut TLoadAsset) -> Self;
}

pub trait AssetFileExtensions {
	fn asset_file_extensions() -> &'static [&'static str];
}
