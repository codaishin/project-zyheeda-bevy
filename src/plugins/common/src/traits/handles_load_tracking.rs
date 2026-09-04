use super::thread_safe::ThreadSafe;
use crate::traits::handles_game_states::{GameState, LoadAssetsExtension, OnStateTransition};
use bevy::{
	app::AppLabel,
	ecs::{schedule::ScheduleLabel, system::ScheduleSystem},
	prelude::*,
};

pub trait HandlesLoadTracking {
	type TLoadAssetState;

	#[must_use]
	fn register_after_load_system(
		load_group: impl LoadGroup<Self::TLoadAssetState>,
	) -> impl RunAfterLoadedInApp;

	#[must_use]
	fn register_load_tracking<T>(
		load_group: impl LoadGroup<Self::TLoadAssetState>,
		progress: impl Progress,
	) -> impl LoadTrackingInApp + LoadTrackingInSubApp
	where
		T: ThreadSafe;

	fn is_loaded(
		load_group: impl LoadGroup<Self::TLoadAssetState>,
	) -> impl IntoSystem<(), bool, (), System: ReadOnlySystem>;

	fn add_loading_systems<M>(
		app: &mut App,
		on_transition: OnStateTransition<impl LoadGroup<Self::TLoadAssetState>>,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	);
}

pub trait LoadGroup<TLoadAssetState>: internal::LoadGroup + ThreadSafe + Clone + Copy {
	fn load_state(&self, load_state: TLoadAssetState) -> LoadAssetsExtension<TLoadAssetState>;

	fn load_state_done(&self) -> GameState;
}

pub trait RunAfterLoadedInApp {
	fn in_app<TMarker>(
		self,
		app: &mut App,
		schedule: impl ScheduleLabel,
		system: impl IntoSystem<(), (), TMarker>,
	);
}

pub trait LoadTrackingInApp {
	fn in_app<TMarker, TLoaded>(
		self,
		app: &mut App,
		all_loaded: impl IntoSystem<(), TLoaded, TMarker>,
	) where
		TLoaded: Into<Loaded> + 'static;
}

pub trait LoadTrackingInSubApp {
	fn in_sub_app<TMarker>(
		self,
		app: &mut App,
		app_label: impl AppLabel,
		all_loaded: impl IntoSystem<(), Loaded, TMarker>,
	);
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Loaded(pub bool);

impl Loaded {
	pub fn when(loaded: bool) -> Self {
		Self(loaded)
	}
}

impl From<bool> for Loaded {
	fn from(loaded: bool) -> Self {
		Loaded(loaded)
	}
}

pub trait Progress: internal::Progress + ThreadSafe + Clone + Copy {
	fn is_processing(&self) -> IsProcessing;
}

pub enum IsProcessing {
	Assets,
	Dependencies,
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct AssetsProgress;

impl Progress for AssetsProgress {
	fn is_processing(&self) -> IsProcessing {
		const { IsProcessing::Assets }
	}
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct DependenciesProgress;

impl Progress for DependenciesProgress {
	fn is_processing(&self) -> IsProcessing {
		const { IsProcessing::Dependencies }
	}
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct LoadingEssentialAssets;

impl<TLoadSteps> LoadGroup<TLoadSteps> for LoadingEssentialAssets {
	fn load_state(&self, load_steps: TLoadSteps) -> LoadAssetsExtension<TLoadSteps> {
		LoadAssetsExtension::LoadEssentials(load_steps)
	}

	fn load_state_done(&self) -> GameState {
		GameState::StartScreen
	}
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct LoadingGame;

impl<TLoadSteps> LoadGroup<TLoadSteps> for LoadingGame {
	fn load_state(&self, load_steps: TLoadSteps) -> LoadAssetsExtension<TLoadSteps> {
		LoadAssetsExtension::Load(load_steps)
	}

	fn load_state_done(&self) -> GameState {
		GameState::Play
	}
}

mod internal {
	use super::*;

	pub trait Progress {}

	impl Progress for AssetsProgress {}
	impl Progress for DependenciesProgress {}

	pub trait LoadGroup {}

	impl LoadGroup for LoadingEssentialAssets {}
	impl LoadGroup for LoadingGame {}
}
