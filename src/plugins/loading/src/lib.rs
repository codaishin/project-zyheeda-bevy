pub mod resources;
pub mod systems;
pub mod traits;

mod asset_loader;

use crate::{
	resources::{group_loaded::GroupLoaded, track::IsDone, uniques::Uniques},
	systems::{
		begin_loading_resource::BeginLoadingResource,
		instantiate_resource::InstantiateResource,
	},
};
use asset_loader::CustomAssetLoader;
use bevy::{app::AppLabel, ecs::schedule::ScheduleLabel, prelude::*};
use common::{prelude::*, tools::path::Path};
use resources::track::Track;
use serde::Deserialize;
use std::{any::type_name, error::Error, fmt::Debug, marker::PhantomData};
use systems::{begin_loading_folder_assets::begin_loading_folder_assets, is_loaded::is_loaded};
use zyheeda_core::prelude::*;

pub struct LoadingPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TGameStates> LoadingPlugin<TGameStates>
where
	TGameStates: ThreadSafe + HandlesGameStates + SystemSetDefinition,
{
	pub fn from_plugins(_: &TGameStates) -> Self {
		Self(PhantomData)
	}

	fn default_transitions(app: &mut App) -> Result<(), TransitionsConfigError> {
		TGameStates::add_activity_transitions(
			app,
			SettableActivity::NewGame,
			always,
			hash_map! { () => ActivityTransition::To(Activity::LoadAssets(LoadActivity::Assets)) },
		)?;

		Ok(())
	}

	fn load_transitions<TLoadGroup>(
		app: &mut App,
		load_assets: Activity,
		load_deps: Activity,
		done: Activity,
	) -> Result<(), TransitionsConfigError>
	where
		TLoadGroup: ThreadSafe,
	{
		TGameStates::add_activity_transitions(
			app,
			load_assets,
			Track::<TLoadGroup, AssetsProgress>::is_done,
			hash_map! { IsDone => ActivityTransition::To(load_deps) },
		)?;

		TGameStates::add_activity_transitions(
			app,
			load_deps,
			Track::<TLoadGroup, DependenciesProgress>::is_done,
			hash_map! { IsDone => ActivityTransition::To(done) },
		)?;

		Ok(())
	}
}

impl<TGameStates> Plugin for LoadingPlugin<TGameStates>
where
	TGameStates: ThreadSafe + HandlesGameStates + SystemSetDefinition,
{
	fn build(&self, app: &mut App) {
		Self::register_load_group::<LoadingEssentialAssets>(app);
		Self::register_load_group::<LoadingGame>(app);

		if let Err(err) = Self::default_transitions(app) {
			panic!("{err}");
		}
	}
}

impl<TGameStates> HandlesLoadTracking for LoadingPlugin<TGameStates>
where
	TGameStates: ThreadSafe + HandlesGameStates + SystemSetDefinition,
{
	fn register_load_group<TLoadGroup>(app: &mut App)
	where
		TLoadGroup: LoadGroup + ThreadSafe,
	{
		let load = TLoadGroup::LOAD_STATE;
		let done = TLoadGroup::LOAD_DONE_STATE;
		let reset = TLoadGroup::load_reset_states();
		let load_assets = Activity::LoadAssets(load);
		let load_deps = Activity::LoadDependencies(load);

		app.add_systems(
			First,
			(
				Track::<TLoadGroup, AssetsProgress>::track_lifetime
					.run_if(TGameStates::in_game_state([load_assets])),
				Track::<TLoadGroup, DependenciesProgress>::track_lifetime
					.run_if(TGameStates::in_game_state([load_deps])),
			),
		);

		for reset in reset {
			TGameStates::add_game_state_systems(
				app,
				OnGameState::Enter(reset),
				GroupLoaded::<TLoadGroup>::remove,
			);
		}

		TGameStates::add_game_state_systems(
			app,
			OnGameState::Enter(load_assets),
			Track::<TLoadGroup, AssetsProgress>::init,
		);
		TGameStates::add_game_state_systems(
			app,
			OnGameState::Exit(load_assets),
			Track::<TLoadGroup, AssetsProgress>::remove,
		);
		TGameStates::add_game_state_systems(
			app,
			OnGameState::Enter(load_deps),
			Track::<TLoadGroup, DependenciesProgress>::init,
		);
		TGameStates::add_game_state_systems(
			app,
			OnGameState::Exit(load_deps),
			Track::<TLoadGroup, DependenciesProgress>::remove,
		);
		TGameStates::add_game_state_systems(
			app,
			OnGameState::Enter(done),
			GroupLoaded::<TLoadGroup>::insert,
		);

		if let Err(err) = Self::load_transitions::<TLoadGroup>(app, load_assets, load_deps, done) {
			panic!("{err}");
		}
	}

	fn register_after_load_system<TLoadGroup>() -> impl RunAfterLoadedInApp
	where
		TLoadGroup: ThreadSafe,
	{
		RegisterAfterLoadSystem(PhantomData::<TLoadGroup>)
	}

	fn register_load_tracking<T, TLoadGroup, TProgress>()
	-> impl LoadTrackingInApp + LoadTrackingInSubApp
	where
		T: 'static,
		TLoadGroup: ThreadSafe + LoadGroup,
		TProgress: Progress + ThreadSafe,
	{
		RegisterLoadTracking(PhantomData::<(T, TGameStates, TLoadGroup, TProgress)>)
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
		let group_loaded = resource_exists::<GroupLoaded<TLoadGroup>>;
		app.add_systems(schedule, system.run_if(group_loaded));
	}
}

struct RegisterLoadTracking<T, TGameState, TLoadGroup, TProgress>(
	PhantomData<(T, TGameState, TLoadGroup, TProgress)>,
);

impl<T, TGameState, TLoadGroup, TProgress> LoadTrackingInApp
	for RegisterLoadTracking<T, TGameState, TLoadGroup, TProgress>
where
	T: 'static,
	TGameState: ThreadSafe + HandlesGameStates + SystemSetDefinition,
	TLoadGroup: ThreadSafe + LoadGroup,
	TProgress: ThreadSafe + Progress,
{
	fn in_app<TMarker, TLoaded>(
		self,
		app: &mut App,
		all_loaded: impl IntoSystem<(), TLoaded, TMarker>,
	) where
		TLoaded: Into<Loaded> + 'static,
	{
		let load = TLoadGroup::LOAD_STATE;
		let state = match TProgress::IS_PROCESSING {
			IsProcessing::Assets => Activity::LoadAssets(load),
			IsProcessing::Dependencies => Activity::LoadDependencies(load),
		};
		let mut uniques = Uniques::mut_from(app);
		let is_unique = uniques.register::<(TLoadGroup, TProgress, T)>().is_unique();

		if !is_unique {
			tracing::error!(
				"Failed to register tracker for '{}': It is already tracked for '{}' in '{}'",
				type_name::<T>(),
				type_name::<TProgress>(),
				type_name::<TLoadGroup>(),
			);
			return;
		}

		app.add_systems(
			Update,
			all_loaded
				.pipe(Track::<TLoadGroup, TProgress>::track_system::<T, TLoaded>)
				.chain()
				.run_if(TGameState::in_game_state([state]))
				.after_plugin(TGameState::SYSTEMS),
		);
	}
}

impl<T, TGameStates, TLoadGroup, TProgress> LoadTrackingInSubApp
	for RegisterLoadTracking<T, TGameStates, TLoadGroup, TProgress>
where
	T: 'static,
	TGameStates: ThreadSafe + HandlesGameStates + SystemSetDefinition,
	TProgress: ThreadSafe + Progress,
	TLoadGroup: ThreadSafe + LoadGroup,
{
	fn in_sub_app<TMarker>(
		self,
		app: &mut App,
		app_label: impl AppLabel,
		all_loaded: impl IntoSystem<(), Loaded, TMarker>,
	) {
		let mut uniques = Uniques::mut_from(app);
		let is_unique = uniques.register::<(TLoadGroup, TProgress, T)>().is_unique();

		if !is_unique {
			tracing::error!(
				"Failed to register tracker for '{}': It is already tracked for '{}' in '{}'",
				type_name::<T>(),
				type_name::<TProgress>(),
				type_name::<TLoadGroup>(),
			);
			return;
		}

		app.sub_app_mut(app_label).add_systems(
			ExtractSchedule,
			all_loaded
				.pipe(Track::<TLoadGroup, TProgress>::track_in_main_world_system::<T>)
				.chain()
				.run_if(Track::<TLoadGroup, TProgress>::main_world_is_processing)
				.after_plugin(TGameStates::SYSTEMS),
		);
	}
}

impl<TDependencies> HandlesCustomAssets for LoadingPlugin<TDependencies> {
	fn register_custom_assets<TAsset, TDto>(app: &mut App)
	where
		TAsset: Asset + TryLoadFrom<TDto> + Clone + std::fmt::Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + TypePath + ThreadSafe,
	{
		app.init_asset::<TAsset>()
			.register_asset_loader(CustomAssetLoader::<TAsset, TDto>::default());
	}
}

impl<TGameStates> HandlesCustomFolderAssets for LoadingPlugin<TGameStates>
where
	TGameStates: ThreadSafe + HandlesGameStates + SystemSetDefinition,
{
	fn register_custom_folder_assets<TAsset, TDto, TLoadGroup>(app: &mut App)
	where
		TAsset: Asset + AssetFolderPath + TryLoadFrom<TDto> + Clone + std::fmt::Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + TypePath + ThreadSafe,
		TLoadGroup: ThreadSafe + LoadGroup,
	{
		Self::register_custom_assets::<TAsset, TDto>(app);
		Self::register_load_tracking::<FolderLoadingOf<TAsset>, TLoadGroup, AssetsProgress>()
			.in_app(app, is_loaded::<TAsset>);

		TGameStates::add_game_state_systems(
			app,
			OnGameState::Enter(Activity::LoadAssets(TLoadGroup::LOAD_STATE)),
			begin_loading_folder_assets::<TAsset, AssetServer>,
		);
	}
}

struct FolderLoadingOf<TAsset>(PhantomData<TAsset>);

impl<TGameStates> HandlesAssetResourceLoading for LoadingPlugin<TGameStates>
where
	TGameStates: ThreadSafe + HandlesGameStates + SystemSetDefinition,
{
	fn register_custom_resource_loading<TResource, TDto, TLoadGroup>(app: &mut App, path: Path)
	where
		TResource: Resource
			+ Asset
			+ Clone
			+ TryLoadFrom<TDto, TInstantiationError: Error + TypePath + ThreadSafe>
			+ Debug,
		for<'a> TDto: Deserialize<'a> + ThreadSafe + TypePath + AssetFileExtensions,
		TLoadGroup: LoadGroup + ThreadSafe,
	{
		let loading = TLoadGroup::LOAD_STATE;
		let loading_done = resource_exists::<TResource>;
		let loading_incomplete = TGameStates::in_game_state([loading]).and_then(not(loading_done));

		Self::register_custom_assets::<TResource, TDto>(app);
		Self::register_load_tracking::<TResource, TLoadGroup, AssetsProgress>()
			.in_app(app, loading_done);

		TGameStates::add_game_state_systems(
			app,
			OnGameState::Enter(loading),
			TResource::begin_loading(path),
		);

		app.add_systems(
			Update,
			TResource::instantiate
				.run_if(loading_incomplete)
				.after_plugin(TGameStates::SYSTEMS),
		);
	}
}
