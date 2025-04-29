use super::thread_safe::ThreadSafe;
use bevy::{
	app::AppLabel,
	ecs::schedule::ScheduleLabel,
	prelude::*,
	state::state::FreelyMutableState,
};

pub trait HandlesLoadTracking {
	fn processing_state<TLoadGroup, TProgress>() -> impl States + Copy
	where
		TLoadGroup: ThreadSafe,
		TProgress: Progress + ThreadSafe;

	fn register_load_group<TLoadGroup>(app: &mut App)
	where
		TLoadGroup: LoadGroup + ThreadSafe;

	#[must_use]
	fn register_after_load_system<TLoadGroup>() -> impl RunAfterLoadedInApp
	where
		TLoadGroup: ThreadSafe;

	#[must_use]
	fn register_load_tracking<T, TLoadGroup, TProgress>()
	-> impl LoadTrackingInApp + LoadTrackingInSubApp
	where
		T: ThreadSafe,
		TLoadGroup: ThreadSafe,
		TProgress: Progress + ThreadSafe;
}

pub trait LoadGroup {
	type TState: FreelyMutableState + Copy;

	const LOAD_STATE: Self::TState;
	const LOAD_DONE_STATE: Self::TState;
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
		schedule: impl ScheduleLabel,
		all_loaded: impl IntoSystem<(), Loaded, TMarker>,
	);
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Loaded(pub bool);

impl From<bool> for Loaded {
	fn from(loaded: bool) -> Self {
		Loaded(loaded)
	}
}

pub trait Progress: internal::Progress {
	const IS_PROCESSING: IsProcessing;
}

pub enum IsProcessing {
	Assets,
	Dependencies,
}

#[derive(Default, Debug, PartialEq)]
pub struct AssetsProgress;

impl Progress for AssetsProgress {
	const IS_PROCESSING: IsProcessing = IsProcessing::Assets;
}

#[derive(Default, Debug, PartialEq)]
pub struct DependenciesProgress;

impl Progress for DependenciesProgress {
	const IS_PROCESSING: IsProcessing = IsProcessing::Dependencies;
}

mod internal {
	use super::*;

	pub trait Progress {}

	impl Progress for AssetsProgress {}
	impl Progress for DependenciesProgress {}
}
