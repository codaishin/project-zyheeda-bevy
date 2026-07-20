use crate::{
	DepthTexture,
	components::{
		camera_labels::{AgentsPass, EffectLightPass, OutlinePass, VisibilityPass, WorldPass},
		post_process_camera::PostProcessCamera,
	},
	observers::insert_render_target::InsertRenderTarget,
	resources::{camera_render_target::CameraRenderTarget, window_size::WindowSize},
};
use bevy::{
	core_pipeline::{Core3d, Core3dSystems, FullscreenShader},
	prelude::*,
	render::{
		RenderApp,
		RenderStartup,
		extract_component::{
			ComponentUniforms,
			DynamicUniformIndex,
			ExtractComponentPlugin,
			UniformComponentPlugin,
		},
		extract_resource::ExtractResourcePlugin,
		render_asset::RenderAssets,
		render_resource::{
			BindGroup,
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
			TextureViewId,
			binding_types::{sampler, texture_2d, uniform_buffer},
		},
		renderer::{RenderContext, RenderDevice, ViewQuery},
		texture::GpuImage,
		view::ViewTarget,
	},
};
use common::{
	errors::{ErrorData, Level},
	systems::log::OnError,
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
					// agents depth
					texture_2d(TextureSampleType::Depth),
					sampler(SamplerBindingType::Comparison),
					// outline depth
					texture_2d(TextureSampleType::Depth),
					sampler(SamplerBindingType::Comparison),
					// screen (post process camera output)
					texture_2d(TextureSampleType::Float { filterable: true }),
					sampler(SamplerBindingType::Filtering),
					// visibility
					texture_2d(TextureSampleType::Float { filterable: true }),
					sampler(SamplerBindingType::Filtering),
					// effect light
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

pub(crate) trait SetupPostProcessPipeline {
	fn setup_post_process_pipeline(&mut self) -> &mut Self;
}

impl SetupPostProcessPipeline for App {
	fn setup_post_process_pipeline(&mut self) -> &mut Self {
		self.add_plugins((
			ExtractComponentPlugin::<PostProcessCamera>::default(),
			UniformComponentPlugin::<PostProcessCamera>::default(),
			ExtractResourcePlugin::<CameraRenderTarget<VisibilityPass>>::default(),
			ExtractResourcePlugin::<CameraRenderTarget<EffectLightPass>>::default(),
		))
		.add_observer(WorldPass::insert_render_target)
		.add_observer(VisibilityPass::insert_render_target)
		.add_observer(EffectLightPass::insert_render_target)
		.add_systems(
			Startup,
			(
				CameraRenderTarget::<WorldPass>::instantiate,
				CameraRenderTarget::<VisibilityPass>::instantiate,
				CameraRenderTarget::<EffectLightPass>::instantiate,
			),
		)
		.add_systems(
			First,
			(
				CameraRenderTarget::<WorldPass>::update_size,
				CameraRenderTarget::<VisibilityPass>::update_size,
				CameraRenderTarget::<EffectLightPass>::update_size,
			)
				.after(WindowSize::update),
		);

		self.sub_app_mut(RenderApp)
			.add_systems(RenderStartup, PostProcessPipeline::init)
			.add_systems(
				Core3d,
				post_process_system
					.pipe(OnError::log)
					.in_set(Core3dSystems::PostProcess),
			);

		self
	}
}

#[derive(Default)]
struct PostProcessBindGroupCache {
	cached: Option<(TextureViewId, BindGroup)>,
}

#[allow(clippy::too_many_arguments)]
fn post_process_system(
	view: ViewQuery<(&ViewTarget, &DynamicUniformIndex<PostProcessCamera>)>,
	pipeline: Option<Res<PostProcessPipeline>>,
	pipeline_cache: Res<PipelineCache>,
	settings: Res<ComponentUniforms<PostProcessCamera>>,
	gpu_images: Res<RenderAssets<GpuImage>>,
	world_depth: Res<DepthTexture<WorldPass>>,
	visibility: Res<CameraRenderTarget<VisibilityPass>>,
	effect_light: Res<CameraRenderTarget<EffectLightPass>>,
	agents_depth: Res<DepthTexture<AgentsPass>>,
	outline_depth: Res<DepthTexture<OutlinePass>>,
	mut cache: Local<PostProcessBindGroupCache>,
	mut render_context: RenderContext,
) -> Result<(), PostProcessError> {
	use RenderPass::*;
	use RenderPipeline::*;

	let (view_target, settings_index) = view.into_inner();

	// Get render pipeline
	let Some(pipeline) = pipeline else {
		return Err(PostProcessError::RenderPipeline(ResourceMissing));
	};
	let Some(render_pipeline) = pipeline_cache.get_render_pipeline(pipeline.pipeline_id) else {
		return Err(PostProcessError::RenderPipeline(CacheMissing));
	};

	// get post process setting
	let Some(settings_binding) = settings.uniforms().binding() else {
		return Err(PostProcessError::UniformBindings);
	};

	// get target textures
	let Some(world_depth_gpu) = gpu_images.get(&world_depth.handle) else {
		return Err(PostProcessError::GPUImage(WorldDepth));
	};
	let Some(visibility_gpu) = gpu_images.get(&visibility.handle) else {
		return Err(PostProcessError::GPUImage(VisibilityRender));
	};
	let Some(effect_light_gpu) = gpu_images.get(&effect_light.handle) else {
		return Err(PostProcessError::GPUImage(EffectLightRender));
	};
	let Some(agents_depth_gpu) = gpu_images.get(&agents_depth.handle) else {
		return Err(PostProcessError::GPUImage(AgentsDepth));
	};
	let Some(outline_depth_gpu) = gpu_images.get(&outline_depth.handle) else {
		return Err(PostProcessError::GPUImage(OutlineDepth));
	};

	let post_process = view_target.post_process_write();
	let bind_group = match &mut cache.cached {
		Some((texture_id, bind_group)) if post_process.source.id() == *texture_id => bind_group,
		cached => {
			let bind_group = render_context.render_device().create_bind_group(
				"post_process_bind_group",
				&pipeline_cache.get_bind_group_layout(&pipeline.layout),
				&BindGroupEntries::sequential((
					&world_depth_gpu.texture_view,
					&world_depth_gpu.sampler,
					&agents_depth_gpu.texture_view,
					&agents_depth_gpu.sampler,
					&outline_depth_gpu.texture_view,
					&outline_depth_gpu.sampler,
					post_process.source,
					&pipeline.sampler,
					&visibility_gpu.texture_view,
					&visibility_gpu.sampler,
					&effect_light_gpu.texture_view,
					&effect_light_gpu.sampler,
					settings_binding.clone(),
				)),
			);

			let (_, bind_group) = cached.insert((post_process.source.id(), bind_group));
			bind_group
		}
	};

	let mut render_pass =
		render_context
			.command_encoder()
			.begin_render_pass(&RenderPassDescriptor {
				label: Some("post_process_pass"),
				color_attachments: &[Some(RenderPassColorAttachment {
					view: post_process.destination,
					depth_slice: None,
					resolve_target: None,
					ops: Operations::default(),
				})],
				..default()
			});

	render_pass.set_pipeline(render_pipeline);
	render_pass.set_bind_group(0, bind_group, &[settings_index.index()]);
	render_pass.draw(0..3, 0..1);

	Ok(())
}

#[derive(Debug, PartialEq)]
enum PostProcessError {
	RenderPipeline(RenderPipeline),
	UniformBindings,
	GPUImage(RenderPass),
}

impl Display for PostProcessError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "could not obtain {self:?}")
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
enum RenderPipeline {
	ResourceMissing,
	CacheMissing,
}

#[derive(Debug, PartialEq)]
enum RenderPass {
	WorldDepth,
	AgentsDepth,
	OutlineDepth,
	VisibilityRender,
	EffectLightRender,
}
