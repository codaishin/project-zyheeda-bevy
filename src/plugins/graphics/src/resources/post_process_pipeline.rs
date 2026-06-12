use crate::{
	DepthTexture,
	components::{
		camera_labels::{OutlinePass, WorldPass},
		post_process_camera::PostProcessCamera,
	},
	resources::camera_render_target::CameraRenderTarget,
};
use bevy::{
	core_pipeline::{FullscreenShader, core_3d::graph::Node3d},
	ecs::query::QueryItem,
	prelude::*,
	render::{
		extract_component::{ComponentUniforms, DynamicUniformIndex},
		render_asset::RenderAssets,
		render_graph::{NodeRunError, RenderGraphContext, RenderLabel, ViewNode},
		render_resource::{
			BindGroupEntries,
			BindGroupLayoutDescriptor,
			BindGroupLayoutEntries,
			CachedRenderPipelineId,
			ColorTargetState,
			ColorWrites,
			FragmentState,
			Operations,
			PipelineCache,
			RenderPassColorAttachment,
			RenderPassDescriptor,
			RenderPipelineDescriptor,
			Sampler,
			SamplerBindingType,
			SamplerDescriptor,
			ShaderStages,
			TextureFormat,
			TextureSampleType,
			binding_types::{sampler, texture_2d, uniform_buffer},
		},
		renderer::{RenderContext, RenderDevice},
		texture::GpuImage,
		view::ViewTarget,
	},
};
use common::{
	error_logger::{ErrorLogger, Log},
	errors::{ErrorData, Level},
	zyheeda_commands::ZyheedaCommands,
};
use std::{fmt::Display, time::Duration};

#[derive(Resource, Debug)]
pub(crate) struct PostProcessPipeline {
	pub(crate) layout: BindGroupLayoutDescriptor,
	pub(crate) sampler: Sampler,
	pub(crate) pipeline_id: CachedRenderPipelineId,
}

impl PostProcessPipeline {
	pub(crate) fn init(
		mut commands: ZyheedaCommands,
		render_device: Res<RenderDevice>,
		asset_server: Res<AssetServer>,
		fullscreen_shader: Res<FullscreenShader>,
		pipeline_cache: Res<PipelineCache>,
	) {
		let layout = BindGroupLayoutDescriptor::new(
			"post_process_bind_group_layout",
			&BindGroupLayoutEntries::sequential(
				ShaderStages::FRAGMENT,
				(
					// world depth
					texture_2d(TextureSampleType::Depth),
					sampler(SamplerBindingType::Comparison),
					// outline depth
					texture_2d(TextureSampleType::Depth),
					sampler(SamplerBindingType::Comparison),
					// screen (camera output)
					texture_2d(TextureSampleType::Float { filterable: true }),
					sampler(SamplerBindingType::Filtering),
					// outline
					texture_2d(TextureSampleType::Float { filterable: true }),
					sampler(SamplerBindingType::Filtering),
					// shader settings
					uniform_buffer::<PostProcessCamera>(true),
				),
			),
		);
		let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
			label: Some("post_process_pipeline".into()),
			layout: vec![layout.clone()],
			vertex: fullscreen_shader.to_vertex_state(),
			fragment: Some(FragmentState {
				shader: asset_server.load(PostProcessCamera::SHADER_PATH),
				targets: vec![Some(ColorTargetState {
					format: TextureFormat::Rgba16Float,
					blend: None,
					write_mask: ColorWrites::ALL,
				})],
				..default()
			}),
			..default()
		});

		commands.insert_resource(PostProcessPipeline {
			layout,
			sampler: render_device.create_sampler(&SamplerDescriptor::default()),
			pipeline_id,
		});
	}
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

		// get target textures
		let Some(gpu_images) = world.get_resource::<RenderAssets<GpuImage>>() else {
			Self::log(MissingResource::RenderAssets);
			return Ok(());
		};
		let Some(world_depth) = world.get_resource::<DepthTexture<WorldPass>>() else {
			Self::log(MissingResource::RenderTargets(RenderPass::WorldDepth));
			return Ok(());
		};
		let Some(world_depth_gpu) = gpu_images.get(&world_depth.handle) else {
			Self::log(MissingDerived::GPUImage(RenderPass::WorldDepth));
			return Ok(());
		};
		let Some(outline) = world.get_resource::<CameraRenderTarget<OutlinePass>>() else {
			Self::log(MissingResource::RenderTargets(RenderPass::OutlineRender));
			return Ok(());
		};
		let Some(outline_gpu) = gpu_images.get(&outline.handle) else {
			Self::log(MissingDerived::GPUImage(RenderPass::OutlineRender));
			return Ok(());
		};
		let Some(outline_depth) = world.get_resource::<DepthTexture<OutlinePass>>() else {
			Self::log(MissingResource::RenderTargets(RenderPass::OutlineDepth));
			return Ok(());
		};
		let Some(outline_depth_gpu) = gpu_images.get(&outline_depth.handle) else {
			Self::log(MissingDerived::GPUImage(RenderPass::OutlineDepth));
			return Ok(());
		};

		let post_process = view_target.post_process_write();
		let bind_group = render_context.render_device().create_bind_group(
			"post_process_bind_group",
			&cache.get_bind_group_layout(&post_process_pipeline.layout),
			&BindGroupEntries::sequential((
				&world_depth_gpu.texture_view,
				&world_depth_gpu.sampler,
				&outline_depth_gpu.texture_view,
				&outline_depth_gpu.sampler,
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

#[derive(Debug, PartialEq)]
enum MissingResource {
	PostProcessPipeline,
	PipeLineCache,
	PostProcessUniforms,
	RenderAssets,
	RenderTargets(RenderPass),
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
	GPUImage(RenderPass),
}

#[derive(Debug, PartialEq)]
enum RenderPass {
	WorldDepth,
	OutlineRender,
	OutlineDepth,
}
