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
		model_render_layers::ModelRenderLayers,
		post_process_camera::PostProcessCamera,
	},
	materials::effect_material::EffectMaterial,
	observers::insert_render_target::InsertRenderTarget,
	resources::post_process_pipeline::{PostProcessLabel, PostProcessNode, PostProcessPipeline},
	system_params::highlight::{HighlightParam, HighlightParamMut},
};
use bevy::{
	asset::RenderAssetUsages,
	core_pipeline::core_3d::graph::{Core3d, Node3d},
	ecs::query::QueryItem,
	image::{ImageCompareFunction, ImageSampler, ImageSamplerDescriptor},
	prelude::*,
	render::{
		RenderApp,
		RenderStartup,
		camera::ExtractedCamera,
		extract_component::{ExtractComponentPlugin, UniformComponentPlugin},
		extract_resource::{ExtractResource, ExtractResourcePlugin},
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
			CommandEncoderDescriptor,
			Extent3d,
			Origin3d,
			PipelineCache,
			TexelCopyTextureInfo,
			TextureAspect,
			TextureDimension,
			TextureFormat,
		},
		renderer::RenderContext,
		texture::GpuImage,
		view::ViewDepthTexture,
	},
};
use common::{
	components::essence::Essence,
	effects::{force::Force, gravity::Gravity, health_damage::HealthDamage},
	states::game_state::LoadingGame,
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
		app.add_plugins(MaterialPlugin::<EffectMaterial>::default())
			.register_derived_component::<Essence, MaterialOverride>()
			.register_shader::<EssenceMaterial>()
			.add_observer(MaterialOverride::update_essence_shader)
			.add_observer(EffectMaterialHandle::add_to::<TPhysics::TSkillContact>)
			.add_observer(EffectMaterialHandle::add_to::<TPhysics::TSkillProjection>)
			.add_systems(
				Update,
				(
					ModelRenderLayers::systems(),
					EffectMaterialHandle::modify_material::<TPhysics, Force>,
					EffectMaterialHandle::modify_material::<TPhysics, Gravity>,
					EffectMaterialHandle::modify_material::<TPhysics, HealthDamage>,
					EffectMaterialHandle::propagate_material,
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
			.add_plugins(ExtractComponentPlugin::<PostProcessCamera>::default())
			.add_plugins(UniformComponentPlugin::<PostProcessCamera>::default())
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
			.add_systems(PostStartup, spawn_cameras.chain())
			.add_systems(
				First,
				(
					WindowSize::update,
					CameraRenderTarget::<WorldPass>::update_size,
					CameraRenderTarget::<OutlinePass>::update_size,
					Depth::<WorldPass>::update_size,
				)
					.chain(),
			);

		app.sub_app_mut(RenderApp)
			.add_systems(RenderStartup, PostProcessPipeline::init)
			.add_render_graph_node::<ViewNodeRunner<PostProcessNode>>(Core3d, PostProcessLabel)
			.add_render_graph_edges(Core3d, PostProcessLabel::EDGES);
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
		depth_pre_pass(app);
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

// FIXME: Cleanup bellow

#[derive(Resource, ExtractResource, Debug, PartialEq, Clone)]
struct Depth<T>
where
	T: ThreadSafe,
{
	handle: Handle<Image>,
	_p: PhantomData<T>,
}

impl<T> Depth<T>
where
	T: ThreadSafe,
{
	fn init(mut c: Commands, mut images: ResMut<Assets<Image>>) {
		let mut image = Image::new_uninit(
			Extent3d::default(),
			TextureDimension::D2,
			TextureFormat::Depth32Float,
			RenderAssetUsages::default(),
		);
		image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
			label: Some("compare sampler".to_owned()),
			compare: Some(ImageCompareFunction::Always),
			..default()
		});

		c.insert_resource(Self {
			handle: images.add(image),
			_p: PhantomData,
		});
	}

	fn update_size(
		mut depth: ResMut<Self>,
		window_size: Res<WindowSize>,
		mut images: ResMut<Assets<Image>>,
	) {
		if !window_size.is_changed() {
			return;
		}

		let width = window_size.width as u32;
		let height = window_size.height as u32;
		let depth_or_array_layers = 1;

		if width == 0 || height == 0 {
			return;
		}

		let Some(mut image) = images.get(&depth.handle).cloned() else {
			return;
		};

		image.resize(Extent3d {
			width,
			height,
			depth_or_array_layers,
		});

		depth.handle = images.add(image);
	}
}

#[derive(RenderLabel, Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct CopyDepthTexturePass;

#[derive(Default)]
struct DepthPrepassNode<T>(PhantomData<T>);

fn depth_pre_pass(app: &mut App) {
	app.add_plugins(ExtractResourcePlugin::<Depth<WorldPass>>::default())
		.add_systems(Startup, Depth::<WorldPass>::init);

	let render_app = app.sub_app_mut(RenderApp);
	render_app.add_render_graph_node::<ViewNodeRunner<DepthPrepassNode<WorldPass>>>(
		Core3d,
		CopyDepthTexturePass,
	);
	render_app.add_render_graph_edges(
		Core3d,
		(
			Node3d::EndPrepasses,
			CopyDepthTexturePass,
			Node3d::MainOpaquePass,
		),
	);
}

impl<T> ViewNode for DepthPrepassNode<T>
where
	T: ThreadSafe,
{
	type ViewQuery = (&'static ExtractedCamera, &'static ViewDepthTexture);

	fn run<'w>(
		&self,
		_: &mut RenderGraphContext,
		render_context: &mut RenderContext<'w>,
		(camera, depth_texture): QueryItem<'w, '_, Self::ViewQuery>,
		world: &'w World,
	) -> Result<(), NodeRunError> {
		if camera.order > 0 {
			return Ok(());
		};

		let depth_image = world.resource::<Depth<T>>();
		let image_assets = world.resource::<RenderAssets<GpuImage>>();
		let Some(image) = image_assets.get(&depth_image.handle) else {
			return Ok(());
		};

		let src_extend = (
			depth_texture.texture.width(),
			depth_texture.texture.height(),
		);
		let dst_extend = (image.size.width, image.size.height);

		// Can happen during window resizing and would crash the pipeline, so we skip here
		if src_extend != dst_extend {
			return Ok(());
		}

		render_context.add_command_buffer_generation_task(move |render_device| {
			let mut command_encoder =
				render_device.create_command_encoder(&CommandEncoderDescriptor {
					label: Some("copy depth to demo texture command encoder"),
				});
			command_encoder.push_debug_group("copy depth to demo texture");

			command_encoder.copy_texture_to_texture(
				TexelCopyTextureInfo {
					texture: &depth_texture.texture,
					mip_level: 0,
					origin: Origin3d::default(),
					aspect: TextureAspect::DepthOnly,
				},
				TexelCopyTextureInfo {
					texture: &image.texture,
					mip_level: 0,
					origin: Origin3d::default(),
					aspect: TextureAspect::DepthOnly,
				},
				image.size,
			);

			command_encoder.pop_debug_group();
			command_encoder.finish()
		});

		Ok(())
	}
}
