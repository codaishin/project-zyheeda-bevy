pub mod resources;
pub mod systems;
pub mod traits;

use bevy::{
	prelude::*,
	render::{render_resource::PipelineCache, RenderApp},
	state::state::FreelyMutableState,
};
use common::traits::{init_resource::InitResource, remove_resource::RemoveResource};
use resources::load_tracker::LoadTracker;
use systems::no_waiting_pipelines::no_waiting_pipelines;
use traits::register_load_tracking::RegisterLoadTrackingInSubApp;

pub struct LoadingPlugin<TState> {
	pub load_state: TState,
}

impl<TState> Plugin for LoadingPlugin<TState>
where
	TState: FreelyMutableState + Copy,
{
	fn build(&self, app: &mut App) {
		app.add_systems(OnEnter(self.load_state), LoadTracker::init)
			.add_systems(OnExit(self.load_state), LoadTracker::remove)
			.register_load_tracking_in_sub_app::<PipelineCache>(
				RenderApp,
				ExtractSchedule,
				no_waiting_pipelines,
			);
	}
}
