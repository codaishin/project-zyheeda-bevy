pub mod resources;
pub mod systems;
pub mod traits;

mod asset_loader;
mod folder_asset_loader;
mod states;

use crate::{
	states::load_state::Load,
	systems::{
		begin_loading_resource::BeginLoadingResource,
		instantiate_resource::InstantiateResource,
	},
};
use asset_loader::CustomAssetLoader;
use bevy::{app::AppLabel, ecs::schedule::ScheduleLabel, prelude::*};
use common::{
	states::transition_to_state,
	systems::log::log_many,
	traits::{
		handles_asset_resource_loading::HandlesAssetResourceLoading,
		handles_custom_assets::{
			AssetFileExtensions,
			AssetFolderPath,
			HandlesCustomAssets,
			HandlesCustomFolderAssets,
			LoadFrom,
		},
		handles_load_tracking::{
			AssetsProgress,
			DependenciesProgress,
			HandlesLoadTracking,
			LoadGroup,
			LoadTrackingInApp,
			LoadTrackingInSubApp,
			Loaded,
			Progress,
			RunAfterLoadedInApp,
		},
		init_resource::InitResource,
		load_asset::Path,
		remove_resource::RemoveResource,
		thread_safe::ThreadSafe,
	},
};
use folder_asset_loader::{FolderAssetLoader, LoadError, LoadResult};
use resources::{alive_assets::AliveAssets, track::Track};
use serde::Deserialize;
use states::load_state::State;
use std::{fmt::Debug, marker::PhantomData};
use systems::{
	begin_loading_folder_assets::begin_loading_folder_assets,
	is_loaded::is_loaded,
	map_load_results::map_load_results,
};

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
	fn build(&self, _: &mut App) {}
}

impl HandlesLoadTracking for LoadingPlugin {
	fn processing_state<TLoadGroup, TProgress>() -> impl States + Copy
	where
		TLoadGroup: ThreadSafe,
		TProgress: Progress + ThreadSafe,
	{
		Load::<TLoadGroup>::processing::<TProgress>()
	}

	fn register_load_group<TLoadGroup>(app: &mut App)
	where
		TLoadGroup: LoadGroup + ThreadSafe,
	{
		let load_assets = Load::<TLoadGroup>::new(State::LoadAssets);
		let load_deps = Load::<TLoadGroup>::new(State::ResolveDependencies);
		let done = Load::<TLoadGroup>::new(State::Done);

		app.init_state::<Load<TLoadGroup>>()
			.add_systems(
				OnEnter(load_assets),
				Track::<TLoadGroup, AssetsProgress>::init,
			)
			.add_systems(
				OnExit(load_assets),
				Track::<TLoadGroup, AssetsProgress>::remove,
			)
			.add_systems(
				OnEnter(load_deps),
				Track::<TLoadGroup, DependenciesProgress>::init,
			)
			.add_systems(
				OnExit(load_deps),
				Track::<TLoadGroup, DependenciesProgress>::remove,
			)
			.add_systems(
				OnEnter(TLoadGroup::LOAD_STATE),
				transition_to_state(load_assets),
			)
			.add_systems(
				Last,
				(
					Track::<TLoadGroup, AssetsProgress>::when_all_done_set(load_deps),
					Track::<TLoadGroup, DependenciesProgress>::when_all_done_set(done),
					Track::<TLoadGroup, DependenciesProgress>::when_all_done_set(
						TLoadGroup::LOAD_DONE_STATE,
					),
				),
			);
	}

	#[must_use]
	fn register_after_load_system<TLoadGroup>() -> impl RunAfterLoadedInApp
	where
		TLoadGroup: ThreadSafe,
	{
		RegisterAfterLoadSystem(PhantomData::<TLoadGroup>)
	}

	#[must_use]
	fn register_load_tracking<T, TLoadGroup, TProgress>()
	-> impl LoadTrackingInApp + LoadTrackingInSubApp
	where
		T: 'static,
		TLoadGroup: ThreadSafe,
		TProgress: Progress + ThreadSafe,
	{
		RegisterLoadTracking(PhantomData::<(T, TLoadGroup, TProgress)>)
	}
}

struct RegisterAfterLoadSystem<TLoadGroup>(PhantomData<TLoadGroup>);

impl<TLoadGroup> RunAfterLoadedInApp for RegisterAfterLoadSystem<TLoadGroup>
where
	TLoadGroup: ThreadSafe,
{
	fn in_app<TMarker>(
		self,
		app: &mut App,
		schedule: impl ScheduleLabel,
		system: impl IntoSystem<(), (), TMarker>,
	) {
		let done = Load::<TLoadGroup>::new(State::Done);
		app.add_systems(schedule, system.run_if(in_state(done)));
	}
}

struct RegisterLoadTracking<T, TLoadGroup, TProgress>(PhantomData<(T, TLoadGroup, TProgress)>);

impl<T, TLoadGroup, TProgress> LoadTrackingInApp for RegisterLoadTracking<T, TLoadGroup, TProgress>
where
	T: 'static,
	TLoadGroup: ThreadSafe,
	TProgress: Progress + ThreadSafe,
{
	fn in_app<TMarker, TLoaded>(
		self,
		app: &mut App,
		all_loaded: impl IntoSystem<(), TLoaded, TMarker>,
	) where
		TLoaded: Into<Loaded> + 'static,
	{
		app.add_systems(
			Update,
			all_loaded
				.pipe(Track::<TLoadGroup, TProgress>::track::<T, TLoaded>)
				.run_if(in_state(Load::<TLoadGroup>::processing::<TProgress>())),
		);
	}
}

impl<T, TLoadGroup, TProgress> LoadTrackingInSubApp
	for RegisterLoadTracking<T, TLoadGroup, TProgress>
where
	T: 'static,
	TProgress: Progress + ThreadSafe,
	TLoadGroup: ThreadSafe,
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
				.pipe(Track::<TLoadGroup, TProgress>::track_in_main_world::<T>)
				.run_if(Track::<TLoadGroup, TProgress>::main_world_is_processing),
		);
	}
}

impl HandlesCustomAssets for LoadingPlugin {
	fn register_custom_assets<TAsset, TDto>(app: &mut App)
	where
		TAsset: Asset + LoadFrom<TDto> + Clone + std::fmt::Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + ThreadSafe,
	{
		app.init_asset::<TAsset>()
			.register_asset_loader(CustomAssetLoader::<TAsset, TDto>::default());
	}
}

impl HandlesCustomFolderAssets for LoadingPlugin {
	fn register_custom_folder_assets<TAsset, TDto, TLoadGroup>(app: &mut App)
	where
		TAsset: Asset + AssetFolderPath + LoadFrom<TDto> + Clone + std::fmt::Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + ThreadSafe,
		TLoadGroup: ThreadSafe,
	{
		LoadingPlugin::register_custom_assets::<TAsset, TDto>(app);
		LoadingPlugin::register_load_tracking::<AliveAssets<TAsset>, TLoadGroup, AssetsProgress>()
			.in_app(app, is_loaded::<TAsset>);

		let load_assets = Load::<TLoadGroup>::new(State::LoadAssets);

		app.init_asset::<LoadResult<TAsset>>()
			.init_resource::<AliveAssets<TAsset>>()
			.register_asset_loader(FolderAssetLoader::<TAsset, TDto>::default())
			.add_systems(
				OnEnter(load_assets),
				begin_loading_folder_assets::<TAsset, AssetServer>,
			)
			.add_systems(
				Update,
				map_load_results::<TAsset, LoadError, AssetServer>
					.pipe(log_many)
					.run_if(in_state(load_assets)),
			);
	}
}

impl HandlesAssetResourceLoading for LoadingPlugin {
	fn register_custom_resource_loading<TResource, TDto, TLoadGroup>(app: &mut App, path: Path)
	where
		TResource: Resource + Asset + Clone + LoadFrom<TDto> + Debug,
		for<'a> TDto: Deserialize<'a> + ThreadSafe + AssetFileExtensions,
		TLoadGroup: LoadGroup + ThreadSafe,
	{
		let loading = TLoadGroup::LOAD_STATE;
		let loading_done = resource_exists::<TResource>;
		let on_begin_load = OnEnter(loading);
		let loading_incomplete = in_state(loading).and(not(loading_done));

		LoadingPlugin::register_custom_assets::<TResource, TDto>(app);
		LoadingPlugin::register_load_tracking::<TResource, TLoadGroup, AssetsProgress>()
			.in_app(app, loading_done);

		app.add_systems(on_begin_load, TResource::begin_loading(path))
			.add_systems(Update, TResource::instantiate.run_if(loading_incomplete));
	}
}
