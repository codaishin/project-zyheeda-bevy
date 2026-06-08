use crate::{PostProcessCamera, resources::post_process_pipeline::PostProcessPipeline};
use bevy::{
	core_pipeline::FullscreenShader,
	prelude::*,
	render::{
		render_resource::{
			BindGroupLayoutDescriptor,
			BindGroupLayoutEntries,
			ColorTargetState,
			ColorWrites,
			FragmentState,
			PipelineCache,
			RenderPipelineDescriptor,
			SamplerBindingType,
			SamplerDescriptor,
			ShaderStages,
			TextureFormat,
			TextureSampleType,
			binding_types::{sampler, texture_2d, uniform_buffer},
		},
		renderer::RenderDevice,
	},
};
use common::zyheeda_commands::ZyheedaCommands;

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
					texture_2d(TextureSampleType::Float { filterable: true }),
					sampler(SamplerBindingType::Filtering),
					texture_2d(TextureSampleType::Float { filterable: true }),
					sampler(SamplerBindingType::Filtering),
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
