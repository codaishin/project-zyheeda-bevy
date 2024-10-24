use crate::{
	folder_asset_loader::{FolderAssetLoader, LoadError, LoadResult},
	resources::AliveAssets,
	states::{AssetLoadState, LoadState},
	systems::{
		begin_loading_folder_assets::begin_loading_folder_assets,
		log::log_many,
		map_load_results::map_load_results,
		set_assets_to_loaded::set_assets_to_loaded,
	},
	traits::{
		asset_file_extensions::AssetFileExtensions,
		asset_folder::AssetFolderPath,
		load_from::LoadFrom,
	},
};
use std::fmt::Debug;

use super::RegisterCustomFolderAssets;
use bevy::{
	app::{App, PostStartup, Update},
	asset::{Asset, AssetApp, AssetServer},
	prelude::{AppExtStates, IntoSystem},
};
use serde::Deserialize;

impl RegisterCustomFolderAssets for App {
	fn register_custom_folder_assets<TAsset, TDto>(&mut self) -> &mut Self
	where
		TAsset: Asset + AssetFolderPath + LoadFrom<TDto> + Clone + Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + Sync + Send + 'static,
	{
		self.insert_state(AssetLoadState::<TAsset>::new(LoadState::Loading))
			.init_asset::<LoadResult<TAsset>>()
			.init_asset::<TAsset>()
			.init_resource::<AliveAssets<TAsset>>()
			.register_asset_loader(FolderAssetLoader::<TAsset, TDto>::default())
			.add_systems(
				PostStartup,
				begin_loading_folder_assets::<TAsset, AssetServer>,
			)
			.add_systems(
				Update,
				(
					map_load_results::<TAsset, LoadError, AssetServer>.pipe(log_many),
					set_assets_to_loaded::<TAsset>,
				),
			)
	}
}
