pub mod resources;
pub mod systems;
pub mod traits;

pub(crate) mod asset_loader;
pub(crate) mod folder_asset_loader;

use asset_loader::CustomAssetLoader;
use bevy::{
	app::AppLabel,
	ecs::schedule::ScheduleLabel,
	prelude::*,
	state::state::FreelyMutableState,
};
use common::{
	states::{game_state::GameState, load_state::LoadState},
	systems::log::log_many,
	traits::{
		init_resource::InitResource,
		register_custom_assets::{
			AssetFileExtensions,
			AssetFolderPath,
			LoadFrom,
			RegisterCustomAssets,
			RegisterCustomFolderAssets,
		},
		register_load_tracking::{
			AssetsProgress,
			DependenciesProgress,
			InApp,
			InSubApp,
			Loaded,
			Progress,
			RegisterLoadTracking,
			SetState,
		},
		remove_resource::RemoveResource,
	},
};
use folder_asset_loader::{FolderAssetLoader, LoadError, LoadResult};
use resources::{alive_assets::AliveAssets, track::Track};
use serde::Deserialize;
use std::marker::PhantomData;
use systems::{
	begin_loading_folder_assets::begin_loading_folder_assets,
	is_loaded::is_loaded,
	is_processing::is_processing,
	map_load_results::map_load_results,
};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
	fn build(&self, app: &mut App) {
		let load_assets = GameState::Loading(LoadState::Assets);
		let load_deps = GameState::Loading(LoadState::Dependencies);

		app.add_systems(OnEnter(load_assets), Track::<AssetsProgress>::init)
			.add_systems(OnExit(load_assets), Track::<AssetsProgress>::remove)
			.add_systems(OnEnter(load_deps), Track::<DependenciesProgress>::init)
			.add_systems(OnExit(load_deps), Track::<DependenciesProgress>::remove);
	}
}

impl RegisterLoadTracking for LoadingPlugin {
	fn register_after_load_system<TMarker>(
		app: &mut App,
		schedule: impl ScheduleLabel,
		system: impl IntoSystem<(), (), TMarker>,
	) {
		app.add_systems(
			schedule,
			system
				.run_if(not(is_processing::<AssetsProgress>))
				.run_if(not(is_processing::<DependenciesProgress>)),
		);
	}

	fn when_done<TProgress>() -> impl SetState
	where
		TProgress: Progress + Sync + Send + 'static,
	{
		SetStateAfter(PhantomData::<TProgress>)
	}

	fn register_load_tracking<T, TProgress>() -> impl InApp + InSubApp
	where
		T: 'static,
		TProgress: Progress + Send + Sync + 'static,
	{
		Register(PhantomData::<(T, TProgress)>)
	}
}

struct Register<T, TProgress>(PhantomData<(T, TProgress)>);

impl<T, TProgress> InApp for Register<T, TProgress>
where
	T: 'static,
	TProgress: Progress + Send + Sync + 'static,
{
	fn in_app<TMarker>(self, app: &mut App, all_loaded: impl IntoSystem<(), Loaded, TMarker>) {
		app.add_systems(
			Update,
			all_loaded
				.pipe(Track::<TProgress>::track::<T>)
				.run_if(is_processing::<TProgress>),
		);
	}
}

impl<T, TProgress> InSubApp for Register<T, TProgress>
where
	T: 'static,
	TProgress: Progress + Send + Sync + 'static,
{
	fn in_sub_app<TMarker>(
		self,
		app: &mut App,
		app_label: impl AppLabel,
		schedule: impl ScheduleLabel,
		all_loaded: impl IntoSystem<(), Loaded, TMarker>,
	) {
		app.sub_app_mut(app_label).add_systems(
			schedule,
			all_loaded
				.pipe(Track::<TProgress>::track_in_main_world::<T>)
				.run_if(Track::<TProgress>::main_world_is_processing),
		);
	}
}

struct SetStateAfter<TProgress>(PhantomData<TProgress>);

impl<TProgress> SetState for SetStateAfter<TProgress>
where
	TProgress: Progress + Sync + Send + 'static,
{
	fn set_state<TState>(self, app: &mut App, state: TState)
	where
		TState: FreelyMutableState + Copy,
	{
		app.add_systems(Last, Track::<TProgress>::when_all_done_set(state));
	}
}

impl RegisterCustomAssets for LoadingPlugin {
	fn register_custom_assets<TAsset, TDto>(app: &mut App)
	where
		TAsset: Asset + LoadFrom<TDto> + Clone + std::fmt::Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + Sync + Send + 'static,
	{
		app.init_asset::<TAsset>()
			.register_asset_loader(CustomAssetLoader::<TAsset, TDto>::default());
	}
}

impl RegisterCustomFolderAssets for LoadingPlugin {
	fn register_custom_folder_assets<TAsset, TDto>(app: &mut App)
	where
		TAsset: Asset + AssetFolderPath + LoadFrom<TDto> + Clone + std::fmt::Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + Sync + Send + 'static,
	{
		LoadingPlugin::register_custom_assets::<TAsset, TDto>(app);
		LoadingPlugin::register_load_tracking::<AliveAssets<TAsset>, AssetsProgress>()
			.in_app(app, is_loaded::<TAsset>);

		app.init_asset::<LoadResult<TAsset>>()
			.init_resource::<AliveAssets<TAsset>>()
			.register_asset_loader(FolderAssetLoader::<TAsset, TDto>::default())
			.add_systems(
				First,
				begin_loading_folder_assets::<TAsset, AssetServer>
					.run_if(resource_added::<Track<AssetsProgress>>),
			)
			.add_systems(
				Update,
				map_load_results::<TAsset, LoadError, AssetServer>
					.pipe(log_many)
					.run_if(is_processing::<AssetsProgress>),
			);
	}
}
