use super::{
	asset_file_extensions::AssetFileExtensions,
	asset_folder::AssetFolderPath,
	load_from::LoadFrom,
};
use bevy::asset::Asset;
use serde::Deserialize;
use std::fmt::Debug;
pub mod app;

pub trait RegisterFolderAssets {
	fn register_folder_assets<TSkill, TDto>(&mut self) -> &mut Self
	where
		TSkill: Asset + AssetFolderPath + LoadFrom<TDto> + Clone + Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + Sync + Send + 'static;
}
