use bevy::{prelude::*, render::render_resource::PipelineCache};
use common::prelude::*;

pub(crate) fn no_waiting_pipelines(pipelines: Res<PipelineCache>) -> Loaded {
	Loaded(pipelines.waiting_pipelines().next().is_none())
}
