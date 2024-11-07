use super::RegisterCustomAssets;
use crate::{
	asset_loader::CustomAssetLoader,
	traits::{asset_file_extensions::AssetFileExtensions, load_from::LoadFrom},
};
use bevy::{
	app::App,
	asset::{Asset, AssetApp},
};
use serde::Deserialize;
use std::fmt::Debug;

impl RegisterCustomAssets for App {
	fn register_custom_assets<TAsset, TDto>(&mut self) -> &mut Self
	where
		TAsset: Asset + LoadFrom<TDto> + Clone + Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + Sync + Send + 'static,
	{
		self.init_asset::<TAsset>()
			.register_asset_loader(CustomAssetLoader::<TAsset, TDto>::default())
	}
}
