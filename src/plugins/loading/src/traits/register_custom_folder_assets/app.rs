use super::RegisterCustomFolderAssets;
use crate::{
	folder_asset_loader::{FolderAssetLoader, LoadError, LoadResult},
	resources::{alive_assets::AliveAssets, track::Track},
	systems::{
		begin_loading_folder_assets::begin_loading_folder_assets,
		is_loaded::is_loaded,
		is_processing::is_processing,
		map_load_results::map_load_results,
	},
	traits::{
		asset_file_extensions::AssetFileExtensions,
		asset_folder::AssetFolderPath,
		load_from::LoadFrom,
		progress::AssetLoadProgress,
		register_load_tracking::RegisterLoadTracking,
	},
};
use bevy::{
	app::{App, First, Update},
	asset::{Asset, AssetApp, AssetServer},
	prelude::{resource_added, IntoSystem, IntoSystemConfigs},
};
use common::systems::log::log_many;
use serde::Deserialize;
use std::fmt::Debug;

impl RegisterCustomFolderAssets for App {
	fn register_custom_folder_assets<TAsset, TDto>(&mut self) -> &mut Self
	where
		TAsset: Asset + AssetFolderPath + LoadFrom<TDto> + Clone + Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + Sync + Send + 'static,
	{
		self.init_asset::<LoadResult<TAsset>>()
			.init_asset::<TAsset>()
			.init_resource::<AliveAssets<TAsset>>()
			.register_asset_loader(FolderAssetLoader::<TAsset, TDto>::default())
			.register_load_tracking::<TAsset, AssetLoadProgress>(is_loaded::<TAsset>)
			.add_systems(
				First,
				begin_loading_folder_assets::<TAsset, AssetServer>
					.run_if(resource_added::<Track<AssetLoadProgress>>),
			)
			.add_systems(
				Update,
				map_load_results::<TAsset, LoadError, AssetServer>
					.pipe(log_many)
					.run_if(is_processing::<AssetLoadProgress>),
			)
	}
}
