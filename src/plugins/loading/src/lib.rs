pub mod resources;
pub mod systems;
pub mod traits;

pub(crate) mod asset_loader;
pub(crate) mod folder_asset_loader;

use bevy::{app::AppLabel, ecs::schedule::ScheduleLabel, prelude::*};
use common::{
	states::{game_state::GameState, load_state::LoadState},
	traits::{
		init_resource::InitResource,
		register_load_tracking::{
			AssetsProgress,
			DependenciesProgress,
			InApp,
			InSubApp,
			Loaded,
			Progress,
			RegisterLoadTracking,
		},
		remove_resource::RemoveResource,
	},
};
use resources::track::Track;
use std::marker::PhantomData;
use systems::is_processing::is_processing;

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
		all_loaded: impl IntoSystem<(), Loaded, TMarker>,
		app_label: impl AppLabel,
		schedule: impl ScheduleLabel,
	) {
		app.sub_app_mut(app_label).add_systems(
			schedule,
			all_loaded
				.pipe(Track::<TProgress>::track_in_main_world::<T>)
				.run_if(Track::<TProgress>::main_world_is_processing),
		);
	}
}
