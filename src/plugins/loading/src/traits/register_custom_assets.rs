mod app;

use super::{asset_file_extensions::AssetFileExtensions, load_from::LoadFrom};
use bevy::asset::Asset;
use serde::Deserialize;
use std::fmt::Debug;

pub trait RegisterCustomAssets {
	fn register_custom_assets<TAsset, TDto>(&mut self) -> &mut Self
	where
		TAsset: Asset + LoadFrom<TDto> + Clone + Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + Sync + Send + 'static;
}
