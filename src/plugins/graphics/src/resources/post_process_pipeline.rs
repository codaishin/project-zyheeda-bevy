use crate::{
	components::{camera_labels::OutlinePass, post_process_camera::PostProcessCamera},
	resources::camera_render_target::CameraRenderTarget,
};
use bevy::{
	core_pipeline::core_3d::graph::Node3d,
	ecs::query::QueryItem,
	prelude::*,
	render::{
		extract_component::{ComponentUniforms, DynamicUniformIndex},
		render_asset::RenderAssets,
		render_graph::{NodeRunError, RenderGraphContext, RenderLabel, ViewNode},
		render_resource::{
			BindGroupEntries,
			BindGroupLayoutDescriptor,
			CachedRenderPipelineId,
			Operations,
			PipelineCache,
			RenderPassColorAttachment,
			RenderPassDescriptor,
			Sampler,
		},
		renderer::RenderContext,
		texture::GpuImage,
		view::ViewTarget,
	},
};
use common::{
	error_logger::{ErrorLogger, Log},
	errors::{ErrorData, Level},
};
use std::{fmt::Display, time::Duration};

#[derive(Resource, Debug)]
pub(crate) struct PostProcessPipeline {
	pub(crate) layout: BindGroupLayoutDescriptor,
	pub(crate) sampler: Sampler,
	pub(crate) pipeline_id: CachedRenderPipelineId,
}

#[derive(RenderLabel, Debug, PartialEq, Eq, Hash, Clone)]
pub(crate) struct PostProcessLabel;

impl PostProcessLabel {
	pub(crate) const EDGES: (Node3d, Self, Node3d) =
		(Node3d::Tonemapping, Self, Node3d::EndMainPassPostProcessing);
}

#[derive(Default, Debug, PartialEq)]
pub(crate) struct PostProcessNode;

impl PostProcessNode {
	fn log(error: impl Into<PostProcessError>) {
		ErrorLogger::GLOBAL.log(error.into());
	}
}

impl ViewNode for PostProcessNode {
	type ViewQuery = (
		&'static ViewTarget,
		&'static DynamicUniformIndex<PostProcessCamera>,
	);

	fn run(
		&self,
		_: &mut RenderGraphContext,
		render_context: &mut RenderContext,
		(view_target, settings_index, ..): QueryItem<Self::ViewQuery>,
		world: &World,
	) -> Result<(), NodeRunError> {
		// Get render pipeline
		let Some(post_process_pipeline) = world.get_resource::<PostProcessPipeline>() else {
			Self::log(MissingResource::PostProcessPipeline);
			return Ok(());
		};
		let Some(cache) = world.get_resource::<PipelineCache>() else {
			Self::log(MissingResource::PipeLineCache);
			return Ok(());
		};
		let Some(pipeline) = cache.get_render_pipeline(post_process_pipeline.pipeline_id) else {
			Self::log(MissingDerived::RenderPipeline);
			return Ok(());
		};

		// get post process setting
		let Some(settings) = world.get_resource::<ComponentUniforms<PostProcessCamera>>() else {
			Self::log(MissingResource::PostProcessUniforms);
			return Ok(());
		};
		let Some(settings_binding) = settings.uniforms().binding() else {
			Self::log(MissingDerived::UniformBindings);
			return Ok(());
		};

		// get outline target texture
		let Some(gpu_images) = world.get_resource::<RenderAssets<GpuImage>>() else {
			Self::log(MissingResource::RenderAssets);
			return Ok(());
		};
		let Some(outline) = world.get_resource::<CameraRenderTarget<OutlinePass>>() else {
			Self::log(MissingResource::RenderTargets);
			return Ok(());
		};
		let Some(outline_gpu) = gpu_images.get(&outline.handle) else {
			Self::log(MissingDerived::GPUImage);
			return Ok(());
		};

		let post_process = view_target.post_process_write();
		let bind_group = render_context.render_device().create_bind_group(
			"post_process_bind_group",
			&cache.get_bind_group_layout(&post_process_pipeline.layout),
			&BindGroupEntries::sequential((
				post_process.source,
				&post_process_pipeline.sampler,
				&outline_gpu.texture_view,
				&outline_gpu.sampler,
				settings_binding.clone(),
			)),
		);

		let mut render_pass = render_context.begin_tracked_render_pass(RenderPassDescriptor {
			label: Some("post_process_pass"),
			color_attachments: &[Some(RenderPassColorAttachment {
				view: post_process.destination,
				depth_slice: None,
				resolve_target: None,
				ops: Operations::default(),
			})],
			depth_stencil_attachment: None,
			timestamp_writes: None,
			occlusion_query_set: None,
		});

		render_pass.set_render_pipeline(pipeline);
		render_pass.set_bind_group(0, &bind_group, &[settings_index.index()]);
		render_pass.draw(0..3, 0..1);

		Ok(())
	}
}

#[derive(Debug, PartialEq)]
enum PostProcessError {
	MissingResource(MissingResource),
	MissingDerived(MissingDerived),
}

#[derive(Debug, PartialEq)]
enum MissingResource {
	PostProcessPipeline,
	PipeLineCache,
	PostProcessUniforms,
	RenderAssets,
	RenderTargets,
}

impl From<MissingResource> for PostProcessError {
	fn from(value: MissingResource) -> Self {
		Self::MissingResource(value)
	}
}

impl From<MissingDerived> for PostProcessError {
	fn from(value: MissingDerived) -> Self {
		Self::MissingDerived(value)
	}
}

#[derive(Debug, PartialEq)]
enum MissingDerived {
	RenderPipeline,
	UniformBindings,
	GPUImage,
}

impl Display for PostProcessError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			PostProcessError::MissingResource(r) => write!(f, "missing resource {r:?}"),
			PostProcessError::MissingDerived(d) => write!(f, "could not obtain {d:?}"),
		}
	}
}

impl ErrorData for PostProcessError {
	fn rate_limit() -> Option<Duration> {
		Some(Duration::from_secs(2))
	}

	fn level(&self) -> Level {
		Level::Warning
	}

	fn label() -> impl std::fmt::Display {
		"Post Process Error"
	}

	fn into_details(self) -> impl std::fmt::Display {
		self
	}
}
