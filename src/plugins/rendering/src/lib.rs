mod systems;

use bevy::{
	app::*,
	render::{render_resource::PipelineCache, ExtractSchedule, RenderApp},
};
use loading::traits::{
	progress::AssetLoadProgress,
	register_load_tracking::RegisterLoadTrackingInSubApp,
};
use systems::no_waiting_pipelines::no_waiting_pipelines;

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
	fn build(&self, app: &mut App) {
		app.register_load_tracking_in_sub_app::<PipelineCache, AssetLoadProgress>(
			RenderApp,
			ExtractSchedule,
			no_waiting_pipelines,
		);
	}
}
