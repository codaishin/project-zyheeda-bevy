pub mod resources;
pub mod systems;
pub mod traits;

mod asset_loader;
mod states;

use crate::{
	resources::{group_loaded::GroupLoaded, track::IsDone, uniques::Uniques},
	states::asset_load_phase::AssetLoadPhase,
	systems::{
		begin_loading_resource::BeginLoadingResource,
		instantiate_resource::InstantiateResource,
	},
};
use asset_loader::CustomAssetLoader;
use bevy::{
	app::AppLabel,
	ecs::{schedule::ScheduleLabel, system::ScheduleSystem},
	prelude::*,
};
use common::{prelude::*, tools::path::Path};
use resources::track::Track;
use serde::Deserialize;
use std::{any::type_name, error::Error, fmt::Debug, marker::PhantomData};
use systems::{begin_loading_folder_assets::begin_loading_folder_assets, is_loaded::is_loaded};
use zyheeda_core::prelude::*;

type AssetsIO = LoadAssetsExtension<AssetLoadPhase>;

const LOAD_ASSETS: GameStateCommandExtended<AssetsIO> =
	GameStateCommandExtended::Extended(LoadAssetsExtension::Load(AssetLoadPhase::Assets));

pub struct LoadingPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TGameStates> LoadingPlugin<TGameStates>
where
	TGameStates: ThreadSafe + HandlesGameStates + SystemSetDefinition,
{
	pub fn from_plugins(_: &TGameStates) -> Self {
		Self(PhantomData)
	}

	fn default_transitions(
		app: &mut App,
	) -> Result<(), TransitionsConfigError<GameStateCommandExtended<AssetsIO>>> {
		TGameStates::TExtended::<AssetsIO>::add_activity_transitions(
			app,
			GameStateCommand::NewGame,
			always,
			hash_map! {
				() => ActivityTransition::To(LOAD_ASSETS)
			},
		)?;

		Ok(())
	}

	fn load_transitions<TLoadGroup>(
		app: &mut App,
		load_assets: GameStateCommandExtended<AssetsIO>,
		load_deps: GameStateCommandExtended<AssetsIO>,
		done: GameStateCommandExtended<AssetsIO>,
	) -> Result<(), TransitionsConfigError<GameStateCommandExtended<AssetsIO>>>
	where
		TLoadGroup: ThreadSafe,
	{
		TGameStates::TExtended::<AssetsIO>::add_activity_transitions(
			app,
			load_assets,
			Track::<TLoadGroup, AssetsProgress>::is_done,
			hash_map! { IsDone => ActivityTransition::To(load_deps) },
		)?;

		TGameStates::TExtended::<AssetsIO>::add_activity_transitions(
			app,
			load_deps,
			Track::<TLoadGroup, DependenciesProgress>::is_done,
			hash_map! { IsDone => ActivityTransition::To(done) },
		)?;

		Ok(())
	}

	fn register_load_group<TLoadGroup>(app: &mut App, load_group: TLoadGroup)
	where
		TLoadGroup: LoadGroup<AssetLoadPhase> + ThreadSafe,
	{
		let load_assets = GameStateCommandExtended::<AssetsIO>::from(
			load_group.load_state(AssetLoadPhase::Assets),
		);
		let load_deps = GameStateCommandExtended::<AssetsIO>::from(
			load_group.load_state(AssetLoadPhase::Dependencies),
		);
		let done = GameStateCommandExtended::<AssetsIO>::from(load_group.load_state_done());

		let is_loading_assets = TGameStates::TExtended::<AssetsIO>::in_game_state([load_assets]);
		let is_loading_deps = TGameStates::TExtended::<AssetsIO>::in_game_state([load_deps]);

		app.add_systems(
			First,
			(
				Track::<TLoadGroup, AssetsProgress>::track_lifetime.run_if(is_loading_assets),
				Track::<TLoadGroup, DependenciesProgress>::track_lifetime.run_if(is_loading_deps),
			),
		);

		TGameStates::TExtended::<AssetsIO>::add_game_state_systems(
			app,
			OnGameState::Enter(load_assets),
			(
				GroupLoaded::<TLoadGroup>::remove,
				Track::<TLoadGroup, AssetsProgress>::init,
			)
				.chain(),
		);
		TGameStates::TExtended::<AssetsIO>::add_game_state_systems(
			app,
			OnGameState::Exit(load_assets),
			Track::<TLoadGroup, AssetsProgress>::remove,
		);
		TGameStates::TExtended::<AssetsIO>::add_game_state_systems(
			app,
			OnGameState::Enter(load_deps),
			Track::<TLoadGroup, DependenciesProgress>::init,
		);
		TGameStates::TExtended::<AssetsIO>::add_game_state_systems(
			app,
			OnGameState::Exit(load_deps),
			Track::<TLoadGroup, DependenciesProgress>::remove,
		);
		TGameStates::TExtended::<AssetsIO>::add_game_state_systems(
			app,
			OnGameState::Enter(done),
			GroupLoaded::<TLoadGroup>::insert,
		);

		if let Err(err) = Self::load_transitions::<TLoadGroup>(app, load_assets, load_deps, done) {
			panic!("{err}");
		}
	}
}

impl<TGameStates> Plugin for LoadingPlugin<TGameStates>
where
	TGameStates: ThreadSafe + HandlesGameStates + SystemSetDefinition,
{
	fn build(&self, app: &mut App) {
		Self::register_load_group(app, LoadingEssentialAssets);
		Self::register_load_group(app, LoadingGame);

		if let Err(err) = Self::default_transitions(app) {
			panic!("{err}");
		}
	}
}

impl<TGameStates> HandlesLoadTracking for LoadingPlugin<TGameStates>
where
	TGameStates: ThreadSafe + HandlesGameStates + SystemSetDefinition,
{
	type TLoadAssetState = AssetLoadPhase;

	fn register_after_load_system(
		load_group: impl LoadGroup<AssetLoadPhase>,
	) -> impl RunAfterLoadedInApp {
		RegisterAfterLoadSystem(load_group)
	}

	fn register_load_tracking<T>(
		load_group: impl LoadGroup<AssetLoadPhase>,
		progress: impl Progress,
	) -> impl LoadTrackingInApp + LoadTrackingInSubApp
	where
		T: 'static,
	{
		RegisterLoadTracking {
			load_group,
			progress,
			_p: PhantomData::<(T, TGameStates)>,
		}
	}

	fn is_loading(
		load_group: impl LoadGroup<AssetLoadPhase>,
	) -> impl IntoSystem<(), bool, (), System: ReadOnlySystem> {
		TGameStates::TExtended::<AssetsIO>::in_game_state([
			load_group.load_state(AssetLoadPhase::Assets),
			load_group.load_state(AssetLoadPhase::Dependencies),
		])
	}

	fn add_loading_systems<M>(
		app: &mut App,
		on_transition: OnGameState<impl LoadGroup<Self::TLoadAssetState>>,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	) {
		let on_state = match on_transition {
			OnGameState::Enter(load_group) => OnGameState::Enter(GameStateCommandExtended::from(
				load_group.load_state(AssetLoadPhase::Assets),
			)),
			OnGameState::Exit(load_group) => OnGameState::Exit(GameStateCommandExtended::from(
				load_group.load_state(AssetLoadPhase::Dependencies),
			)),
		};

		TGameStates::TExtended::<AssetsIO>::add_game_state_systems(app, on_state, systems);
	}
}

struct RegisterAfterLoadSystem<TLoadGroup>(TLoadGroup);

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

struct RegisterLoadTracking<T, TGameState, TLoadGroup, TProgress> {
	load_group: TLoadGroup,
	progress: TProgress,
	_p: PhantomData<(T, TGameState)>,
}

impl<T, TGameState, TLoadGroup, TProgress> LoadTrackingInApp
	for RegisterLoadTracking<T, TGameState, TLoadGroup, TProgress>
where
	T: 'static,
	TGameState: ThreadSafe + HandlesGameStates + SystemSetDefinition,
	TLoadGroup: ThreadSafe + LoadGroup<AssetLoadPhase>,
	TProgress: ThreadSafe + Progress,
{
	fn in_app<TMarker, TLoaded>(
		self,
		app: &mut App,
		all_loaded: impl IntoSystem<(), TLoaded, TMarker>,
	) where
		TLoaded: Into<Loaded> + 'static,
	{
		let mut uniques = Uniques::mut_from(app);

		if !uniques.register::<(TLoadGroup, TProgress, T)>().is_unique() {
			tracing::error!(
				"Failed to register tracker for '{}': It is already tracked for '{}' in '{}'",
				type_name::<T>(),
				type_name::<TProgress>(),
				type_name::<TLoadGroup>(),
			);
			return;
		}

		let phase = AssetLoadPhase::from(self.progress.is_processing());
		let load_assets =
			GameStateCommandExtended::<AssetsIO>::from(self.load_group.load_state(phase));
		let is_loading_assets = TGameState::TExtended::<AssetsIO>::in_game_state([load_assets]);

		app.add_systems(
			Update,
			all_loaded
				.pipe(Track::<TLoadGroup, TProgress>::track_system::<T, TLoaded>)
				.chain()
				.run_if(is_loading_assets)
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
	TLoadGroup: ThreadSafe + LoadGroup<AssetLoadPhase>,
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
	type TLoadAssetState = AssetLoadPhase;

	fn register_custom_folder_assets<TAsset, TDto>(
		app: &mut App,
		load_group: impl LoadGroup<AssetLoadPhase>,
	) where
		TAsset: Asset + AssetFolderPath + TryLoadFrom<TDto> + Clone + std::fmt::Debug,
		for<'a> TDto: Deserialize<'a> + AssetFileExtensions + TypePath + ThreadSafe,
	{
		Self::register_custom_assets::<TAsset, TDto>(app);
		Self::register_load_tracking::<FolderLoadingOf<TAsset>>(load_group, AssetsProgress)
			.in_app(app, is_loaded::<TAsset>);

		TGameStates::TExtended::<AssetsIO>::add_game_state_systems(
			app,
			OnGameState::Enter(LOAD_ASSETS),
			begin_loading_folder_assets::<TAsset, AssetServer>,
		);
	}
}

struct FolderLoadingOf<TAsset>(PhantomData<TAsset>);

impl<TGameStates> HandlesAssetResourceLoading for LoadingPlugin<TGameStates>
where
	TGameStates: ThreadSafe + HandlesGameStates + SystemSetDefinition,
{
	type TLoadAssetState = AssetLoadPhase;

	fn register_custom_resource_loading<TResource, TDto>(
		app: &mut App,
		load_group: impl LoadGroup<AssetLoadPhase>,
		path: Path,
	) where
		TResource: Resource
			+ Asset
			+ Clone
			+ TryLoadFrom<TDto, TInstantiationError: Error + TypePath + ThreadSafe>
			+ Debug,
		for<'a> TDto: Deserialize<'a> + ThreadSafe + TypePath + AssetFileExtensions,
	{
		let loading = GameStateCommandExtended::<AssetsIO>::from(
			load_group.load_state(AssetLoadPhase::Assets),
		);
		let loading_done = resource_exists::<TResource>;
		let loading_incomplete = TGameStates::TExtended::<AssetsIO>::in_game_state([loading])
			.and_then(not(loading_done));

		Self::register_custom_assets::<TResource, TDto>(app);
		Self::register_load_tracking::<TResource>(load_group, AssetsProgress)
			.in_app(app, loading_done);

		TGameStates::TExtended::<AssetsIO>::add_game_state_systems(
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
