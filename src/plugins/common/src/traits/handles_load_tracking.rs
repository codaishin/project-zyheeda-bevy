use bevy::{
	app::AppLabel,
	ecs::schedule::ScheduleLabel,
	prelude::*,
	state::state::FreelyMutableState,
};

pub trait HandlesLoadTracking {
	fn processing_state<TProgress>() -> impl States + Copy
	where
		TProgress: Progress + Send + Sync + 'static;

	fn register_after_load_system<TMarker>(
		app: &mut App,
		schedule: impl ScheduleLabel,
		system: impl IntoSystem<(), (), TMarker>,
	);

	#[must_use]
	fn begin_loading_on<TState>(app: &mut App, state: TState) -> impl OnLoadingDone
	where
		TState: States + Copy;

	fn register_load_tracking<T, TProgress>() -> impl InApp + InSubApp
	where
		T: 'static,
		TProgress: Progress + Send + Sync + 'static;
}

pub trait InApp {
	fn in_app<TMarker>(self, app: &mut App, all_loaded: impl IntoSystem<(), Loaded, TMarker>);
}

pub trait InSubApp {
	fn in_sub_app<TMarker>(
		self,
		app: &mut App,
		app_label: impl AppLabel,
		schedule: impl ScheduleLabel,
		all_loaded: impl IntoSystem<(), Loaded, TMarker>,
	);
}

pub trait OnLoadingDone {
	fn when_done_set<TState>(self, state: TState)
	where
		TState: FreelyMutableState + Copy;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Loaded(pub bool);

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
