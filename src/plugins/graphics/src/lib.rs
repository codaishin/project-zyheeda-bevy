mod components;
mod materials;
mod observers;
mod resources;
mod system_params;
mod systems;
mod traits;

use crate::{
	components::{
		camera_labels::OutlinePass,
		child_meshes::ChildMeshOf,
		model_render_layers::ModelRenderLayers,
	},
	materials::effect_material::EffectMaterial,
	observers::insert_render_target::InsertRenderTarget,
	system_params::highlight::{HighlightParam, HighlightParamMut},
};
use bevy::{
	core_pipeline::{
		FullscreenShader,
		core_3d::graph::{Core3d, Node3d},
	},
	ecs::query::QueryItem,
	prelude::*,
	render::{
		RenderApp,
		RenderStartup,
		extract_component::{
			ComponentUniforms,
			DynamicUniformIndex,
			ExtractComponent,
			ExtractComponentPlugin,
			UniformComponentPlugin,
		},
		extract_resource::ExtractResourcePlugin,
		render_asset::RenderAssets,
		render_graph::{
			NodeRunError,
			RenderGraphContext,
			RenderGraphExt,
			RenderLabel,
			ViewNode,
			ViewNodeRunner,
		},
		render_resource::{
			binding_types::{sampler, texture_2d, uniform_buffer},
			*,
		},
		renderer::{RenderContext, RenderDevice},
		texture::GpuImage,
		view::ViewTarget,
	},
};
use common::{
	components::essence::Essence,
	effects::{force::Force, gravity::Gravity, health_damage::HealthDamage},
	states::game_state::LoadingGame,
	systems::link::to_target::LinkToTarget,
	traits::{
		after_plugin::AfterPlugin,
		handles_graphics::{FirstPassCamera, HandlesGraphics, UiCamera, WorldCameras},
		handles_load_tracking::{AssetsProgress, HandlesLoadTracking, LoadTrackingInSubApp},
		handles_physics::HandlesAllPhysicalEffects,
		handles_saving::HandlesSaving,
		handles_skill_physics::HandlesSkillPhysics,
		register_derived_component::RegisterDerivedComponent,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use components::{
	camera_labels::{CompositePass, SceneCamera, Ui, WorldPass},
	effect_material_handle::EffectMaterialHandle,
	material_override::MaterialOverride,
};
use materials::essence_material::EssenceMaterial;
use resources::{camera_render_target::CameraRenderTarget, window_size::WindowSize};
use std::{hash::Hash, marker::PhantomData};
use systems::{no_waiting_pipelines::no_waiting_pipelines, spawn_cameras::spawn_cameras};

#[cfg(not(feature = "debug-utils"))]
use components::no_debug_cam::NoDebugCam;

pub struct GraphicsPlugin<TDebugCam, TDependencies> {
	debug_cam: fn() -> TDebugCam,
	_p: PhantomData<TDependencies>,
}

#[cfg(not(feature = "debug-utils"))]
impl<TLoading, TSavegame, TPhysics> GraphicsPlugin<NoDebugCam, (TLoading, TSavegame, TPhysics)>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + SystemSetDefinition + HandlesAllPhysicalEffects + HandlesSkillPhysics,
{
	pub fn from_plugins(_: &TLoading, _: &TSavegame, _: &TPhysics) -> Self {
		Self {
			debug_cam: || NoDebugCam,
			_p: PhantomData,
		}
	}
}

impl<TDebugCam, TLoading, TSavegame, TPhysics>
	GraphicsPlugin<TDebugCam, (TLoading, TSavegame, TPhysics)>
where
	TDebugCam: Component,
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + SystemSetDefinition + HandlesAllPhysicalEffects + HandlesSkillPhysics,
{
	#[cfg(feature = "debug-utils")]
	pub fn new(debug_cam: fn() -> TDebugCam, _: &TLoading, _: &TSavegame, _: &TPhysics) -> Self {
		Self {
			debug_cam,
			_p: PhantomData,
		}
	}

	fn track_render_pipeline_ready(app: &mut App) {
		TLoading::register_load_tracking::<PipelineCache, LoadingGame, AssetsProgress>()
			.in_sub_app(app, RenderApp, ExtractSchedule, no_waiting_pipelines);
	}

	fn shading(app: &mut App) {
		type UnlinkedMeshes = (Added<Mesh3d>, Without<ChildMeshOf>);

		app.add_plugins(MaterialPlugin::<EffectMaterial>::default())
			.register_derived_component::<Essence, MaterialOverride>()
			.register_shader::<EssenceMaterial>()
			.add_observer(MaterialOverride::update_essence_shader)
			.add_observer(EffectMaterialHandle::add_to::<TPhysics::TSkillContact>)
			.add_observer(EffectMaterialHandle::add_to::<TPhysics::TSkillProjection>)
			.add_systems(
				Update,
				(
					UnlinkedMeshes::link_to::<ModelRenderLayers, ChildMeshOf>,
					EffectMaterialHandle::modify_material::<TPhysics, Force>,
					EffectMaterialHandle::modify_material::<TPhysics, Gravity>,
					EffectMaterialHandle::modify_material::<TPhysics, HealthDamage>,
					EffectMaterialHandle::propagate_material,
					ModelRenderLayers::populate_missing_with(WorldPass),
					ModelRenderLayers::propagate_layers,
				)
					.chain()
					.after_plugin(TPhysics::SYSTEMS),
			);
	}

	fn cameras(&self, app: &mut App) {
		app.register_required_components::<SceneCamera, TSavegame::TSaveEntityMarker>();
		TSavegame::register_savable_component::<WorldPass>(app);
		TSavegame::register_savable_component::<OutlinePass>(app);
		TSavegame::register_savable_component::<CompositePass>(app);
		TSavegame::register_savable_component::<Ui>(app);

		app.init_resource::<WindowSize>()
			.add_plugins(ExtractResourcePlugin::<CameraRenderTarget<OutlinePass>>::default())
			.add_plugins(ExtractComponentPlugin::<OutlineSettings>::default())
			.add_plugins(UniformComponentPlugin::<OutlineSettings>::default())
			.register_required_components_with::<Ui, TDebugCam>(self.debug_cam)
			.add_observer(WorldPass::insert_render_target)
			.add_observer(OutlinePass::insert_render_target)
			.add_systems(
				Startup,
				(
					CameraRenderTarget::<WorldPass>::instantiate,
					CameraRenderTarget::<OutlinePass>::instantiate,
				),
			)
			.add_systems(PostStartup, spawn_cameras)
			.add_systems(
				First,
				(
					WindowSize::update,
					CameraRenderTarget::<WorldPass>::update_size,
					CameraRenderTarget::<OutlinePass>::update_size,
				)
					.chain(),
			);

		let Some(render_app) = app.get_sub_app_mut(RenderApp) else {
			// FIXME: ERROR?
			return;
		};

		render_app
			.add_systems(RenderStartup, init_post_process_pipeline)
			.add_render_graph_node::<ViewNodeRunner<PostProcessNode>>(Core3d, PostProcessLabel)
			.add_render_graph_edges(
				Core3d,
				(
					Node3d::Tonemapping,
					PostProcessLabel,
					Node3d::EndMainPassPostProcessing,
				),
			);
	}
}

impl<TDebugCam, TLoading, TSavegame, TPhysics> Plugin
	for GraphicsPlugin<TDebugCam, (TLoading, TSavegame, TPhysics)>
where
	TDebugCam: Component,
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe + SystemSetDefinition + HandlesAllPhysicalEffects + HandlesSkillPhysics,
{
	fn build(&self, app: &mut App) {
		Self::track_render_pipeline_ready(app);
		Self::shading(app);
		self.cameras(app);
	}
}

trait RegisterShader {
	fn register_shader<TMaterial>(&mut self) -> &mut Self
	where
		TMaterial: Material,
		TMaterial::Data: PartialEq + Eq + Hash + Clone;
}

impl RegisterShader for App {
	fn register_shader<TMaterial>(&mut self) -> &mut Self
	where
		TMaterial: Material,
		TMaterial::Data: PartialEq + Eq + Hash + Clone,
	{
		if self.is_plugin_added::<MaterialPlugin<TMaterial>>() {
			return self;
		}

		self.add_plugins(MaterialPlugin::<TMaterial>::default())
	}
}

impl<TDebugCam, TDependencies> UiCamera for GraphicsPlugin<TDebugCam, TDependencies> {
	type TUiCamera = Ui;
}

impl<TDebugCam, TDependencies> FirstPassCamera for GraphicsPlugin<TDebugCam, TDependencies> {
	type TFirstPassCamera = WorldPass;
}

impl<TDebugCam, TDependencies> WorldCameras for GraphicsPlugin<TDebugCam, TDependencies> {
	type TWorldCameras = SceneCamera;
}

impl<TDebugCam, TDependencies> HandlesGraphics for GraphicsPlugin<TDebugCam, TDependencies> {
	type THighlight = HighlightParam<'static, 'static>;
	type THighlightMut = HighlightParamMut<'static, 'static>;
}

// FIXME:: MOVE STUFF BELLOW TO PROPER MODULES

const SHADER_ASSET_PATH: &str = "shaders/post_processing.wgsl";

#[derive(Component, Default, Clone, Copy, ExtractComponent, ShaderType)]
struct OutlineSettings {
	color: LinearRgba,
}

#[derive(Resource)]
struct PostProcessPipeline {
	layout: BindGroupLayoutDescriptor,
	sampler: Sampler,
	pipeline_id: CachedRenderPipelineId,
}

#[derive(Debug, Hash, PartialEq, Eq, Clone, RenderLabel)]
struct PostProcessLabel;

#[derive(Default)]
struct PostProcessNode;

impl ViewNode for PostProcessNode {
	type ViewQuery = (
		&'static ViewTarget,
		&'static DynamicUniformIndex<OutlineSettings>,
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
		let Some(settings) = world.get_resource::<ComponentUniforms<OutlineSettings>>() else {
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

fn init_post_process_pipeline(
	mut commands: Commands,
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
				uniform_buffer::<OutlineSettings>(true),
			),
		),
	);
	let pipeline_id = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
		label: Some("post_process_pipeline".into()),
		layout: vec![layout.clone()],
		vertex: fullscreen_shader.to_vertex_state(),
		fragment: Some(FragmentState {
			shader: asset_server.load(SHADER_ASSET_PATH),
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
