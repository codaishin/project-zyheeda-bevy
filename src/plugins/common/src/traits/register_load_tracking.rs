use bevy::{
	app::AppLabel,
	ecs::schedule::ScheduleLabel,
	prelude::*,
	state::state::FreelyMutableState,
};

pub trait RegisterLoadTracking {
	fn register_after_load_system<TMarker>(
		app: &mut App,
		schedule: impl ScheduleLabel,
		system: impl IntoSystem<(), (), TMarker>,
	);

	fn when_done<TProgress>() -> impl SetState
	where
		TProgress: Progress + Sync + Send + 'static;

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

pub trait SetState {
	fn set_state<TState>(self, app: &mut App, state: TState)
	where
		TState: FreelyMutableState + Copy;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Loaded(pub bool);

pub trait Progress: internal::Progress {}

impl<T> Progress for T where T: internal::Progress {}

#[derive(Default, Debug, PartialEq)]
pub struct AssetsProgress;

#[derive(Default, Debug, PartialEq)]
pub struct DependenciesProgress;

mod internal {
	use super::*;

	pub trait Progress {}

	impl Progress for AssetsProgress {}
	impl Progress for DependenciesProgress {}
}
