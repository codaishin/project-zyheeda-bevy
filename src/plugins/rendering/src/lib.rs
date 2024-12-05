mod systems;

use bevy::{
	app::*,
	render::{render_resource::PipelineCache, ExtractSchedule, RenderApp},
};
use common::traits::handles_load_tracking::{AssetsProgress, HandlesLoadTracking, InSubApp};
use std::marker::PhantomData;
use systems::no_waiting_pipelines::no_waiting_pipelines;

pub struct RenderingPlugin<TLoading>(PhantomData<TLoading>);

impl<TLoading> RenderingPlugin<TLoading>
where
	TLoading: Plugin + HandlesLoadTracking,
{
	pub fn depends_on(_: &TLoading) -> Self {
		Self(PhantomData)
	}
}

impl<TLoading> Plugin for RenderingPlugin<TLoading>
where
	TLoading: Plugin + HandlesLoadTracking,
{
	fn build(&self, app: &mut App) {
		TLoading::register_load_tracking::<PipelineCache, AssetsProgress>().in_sub_app(
			app,
			RenderApp,
			ExtractSchedule,
			no_waiting_pipelines,
		);
	}
}
