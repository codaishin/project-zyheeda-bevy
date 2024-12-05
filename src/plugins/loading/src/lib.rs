pub mod resources;
pub mod systems;
pub mod traits;

mod asset_loader;
mod folder_asset_loader;
mod states;

use crate::states::load_state::LoadState;
use asset_loader::CustomAssetLoader;
use bevy::{
	app::AppLabel,
	ecs::schedule::ScheduleLabel,
	prelude::*,
	state::state::FreelyMutableState,
};
use common::{
	states::transition_to_state,
	systems::log::log_many,
	traits::{
		handles_load_tracking::{
			AssetsProgress,
			DependenciesProgress,
			HandlesLoadTracking,
			InApp,
			InSubApp,
			Loaded,
			OnLoadingDone,
			Progress,
		},
		init_resource::InitResource,
		register_custom_assets::{
			AssetFileExtensions,
			AssetFolderPath,
			LoadFrom,
			RegisterCustomAssets,
			RegisterCustomFolderAssets,
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
	map_load_results::map_load_results,
};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
	fn build(&self, app: &mut App) {
		let load_assets = LoadState::LoadAssets;
		let load_deps = LoadState::ResolveDependencies;
		let done = LoadState::Done;

		app.init_state::<LoadState>()
			.add_systems(OnEnter(load_assets), Track::<AssetsProgress>::init)
			.add_systems(OnExit(load_assets), Track::<AssetsProgress>::remove)
			.add_systems(OnEnter(load_deps), Track::<DependenciesProgress>::init)
			.add_systems(OnExit(load_deps), Track::<DependenciesProgress>::remove)
			.add_systems(
				Last,
				(
					Track::<AssetsProgress>::when_all_done_set(load_deps),
					Track::<DependenciesProgress>::when_all_done_set(done),
				),
			);
	}
}

impl HandlesLoadTracking for LoadingPlugin {
	fn processing_state<TProgress>() -> impl States + Copy
	where
		TProgress: Progress + Send + Sync + 'static,
	{
		LoadState::processing::<TProgress>()
	}

	fn register_after_load_system<TMarker>(
		app: &mut App,
		schedule: impl ScheduleLabel,
		system: impl IntoSystem<(), (), TMarker>,
	) {
		app.add_systems(schedule, system.run_if(in_state(LoadState::Done)));
	}

	fn begin_loading_on<TState>(app: &mut App, state: TState) -> impl OnLoadingDone
	where
		TState: States + Copy,
	{
		app.add_systems(OnEnter(state), transition_to_state(LoadState::LoadAssets));
		SetStateWhenDone(app)
	}

	fn register_load_tracking<T, TProgress>() -> impl InApp + InSubApp
	where
		T: 'static,
		TProgress: Progress + Send + Sync + 'static,
	{
		Register(PhantomData::<(T, TProgress)>)
	}
}

struct SetStateWhenDone<'a>(&'a mut App);

impl OnLoadingDone for SetStateWhenDone<'_> {
	fn when_done_set<TState>(self, state: TState)
	where
		TState: FreelyMutableState + Copy,
	{
		let Self(app) = self;
		app.add_systems(
			Last,
			Track::<DependenciesProgress>::when_all_done_set(state),
		);
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
				.run_if(in_state(LoadState::processing::<TProgress>())),
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
				OnEnter(LoadState::LoadAssets),
				begin_loading_folder_assets::<TAsset, AssetServer>,
			)
			.add_systems(
				Update,
				map_load_results::<TAsset, LoadError, AssetServer>
					.pipe(log_many)
					.run_if(in_state(LoadState::LoadAssets)),
			);
	}
}
