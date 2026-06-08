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

impl ViewNode for PostProcessNode {
	type ViewQuery = (
		&'static ViewTarget,
		&'static DynamicUniformIndex<PostProcessCamera>,
	);

	fn run(
		&self,
		_: &mut RenderGraphContext,
		render_context: &mut RenderContext,
		(view_target, settings_index): QueryItem<Self::ViewQuery>,
		world: &World,
	) -> Result<(), NodeRunError> {
		// Get render pipeline
		let Some(post_process_pipeline) = world.get_resource::<PostProcessPipeline>() else {
			// FIXME: ERROR?
			return Ok(());
		};
		let Some(cache) = world.get_resource::<PipelineCache>() else {
			// FIXME: ERROR?
			return Ok(());
		};
		let Some(pipeline) = cache.get_render_pipeline(post_process_pipeline.pipeline_id) else {
			// FIXME: ERROR?
			return Ok(());
		};

		// get `PostProcessSettings` as bindings
		let Some(settings) = world.get_resource::<ComponentUniforms<PostProcessCamera>>() else {
			// FIXME: ERROR?
			return Ok(());
		};
		let Some(settings_binding) = settings.uniforms().binding() else {
			// FIXME: ERROR?
			return Ok(());
		};

		// get outline target texture
		let Some(gpu_images) = world.get_resource::<RenderAssets<GpuImage>>() else {
			// FIXME: ERROR?
			return Ok(());
		};
		let Some(outline) = world.get_resource::<CameraRenderTarget<OutlinePass>>() else {
			// FIXME: ERROR?
			return Ok(());
		};
		let Some(outline_gpu) = gpu_images.get(&outline.handle) else {
			// FIXME: ERROR?
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
