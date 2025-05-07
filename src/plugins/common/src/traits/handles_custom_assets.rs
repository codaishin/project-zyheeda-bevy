use super::{
	load_asset::{LoadAsset, Path},
	thread_safe::ThreadSafe,
};
use bevy::prelude::*;
use serde::Deserialize;
use std::{error::Error, fmt::Debug};

pub trait HandlesCustomAssets {
	fn register_custom_assets<TAsset, TDto>(app: &mut App)
	where
		TAsset: Asset + TryLoadFrom<TDto> + Clone + Debug,
		TAsset::TInstantiationError: Error + TypePath + ThreadSafe,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + ThreadSafe;
}

pub trait HandlesCustomFolderAssets {
	fn register_custom_folder_assets<TAsset, TDto, TLoadGroup>(app: &mut App)
	where
		TAsset: Asset + AssetFolderPath + TryLoadFrom<TDto> + Clone + Debug,
		TAsset::TInstantiationError: Error + TypePath + ThreadSafe,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + ThreadSafe,
		TLoadGroup: ThreadSafe;
}

pub trait AssetFolderPath {
	fn asset_folder_path() -> Path;
}

pub trait TryLoadFrom<TFrom>: Sized {
	type TInstantiationError;

	fn try_load_from<TLoadAsset>(
		from: TFrom,
		asset_server: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset;
}

pub trait AssetFileExtensions {
	fn asset_file_extensions() -> &'static [&'static str];
}
