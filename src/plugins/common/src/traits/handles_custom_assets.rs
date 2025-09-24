use super::{
	load_asset::{LoadAsset, Path},
	thread_safe::ThreadSafe,
};
use crate::errors::Unreachable;
use bevy::{app::App, asset::Asset, reflect::TypePath};
use serde::Deserialize;
use std::{error::Error, fmt::Debug};

pub trait HandlesCustomAssets {
	fn register_custom_assets<TAsset, TDto>(app: &mut App)
	where
		TAsset: Asset + TryLoadFrom<TDto> + Clone + Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + ThreadSafe;
}

pub trait HandlesCustomFolderAssets {
	fn register_custom_folder_assets<TAsset, TDto, TLoadGroup>(
		app: &mut App,
		on_load_error: OnLoadError,
	) where
		TAsset: Asset + AssetFolderPath + TryLoadFrom<TDto> + Clone + Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + ThreadSafe,
		TLoadGroup: ThreadSafe;
}

pub enum OnLoadError {
	SkipAsset,
	Panic,
}

pub trait AssetFolderPath {
	fn asset_folder_path() -> Path;
}

pub trait TryLoadFrom<TFrom>: Sized {
	type TInstantiationError: Error + TypePath + ThreadSafe;

	fn try_load_from<TLoadAsset>(
		from: TFrom,
		asset_server: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset;
}

impl<T> TryLoadFrom<T> for T {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(from: T, _: &mut TLoadAsset) -> Result<Self, Unreachable>
	where
		TLoadAsset: LoadAsset,
	{
		Ok(from)
	}
}

pub trait AssetFileExtensions {
	fn asset_file_extensions() -> &'static [&'static str];
}
