mod components;
mod materials;
mod observers;
mod resources;
mod system_params;
mod systems;
mod traits;

use crate::{
	components::{
		camera_labels::{AgentsPass, OutlinePass, VisibilityPass},
		model_render_layers::ModelRenderLayers,
		post_process_camera::PostProcessCamera,
		roles::{Enemy, Player},
	},
	materials::effect_material::EffectMaterial,
	observers::insert_render_target::InsertRenderTarget,
	resources::{
		depth_texture::{CopyDepthTextureNode, DepthTexture, DepthTextureLabel},
		post_process_pipeline::{PostProcessLabel, PostProcessNode, PostProcessPipeline},
	},
	system_params::{
		highlight::{HighlightParam, HighlightParamMut},
		lights::RolesParamMut,
	},
};
use bevy::{
	core_pipeline::core_3d::graph::Core3d,
	prelude::*,
	render::{
		RenderApp,
		RenderStartup,
		extract_component::{ExtractComponentPlugin, UniformComponentPlugin},
		extract_resource::ExtractResourcePlugin,
		render_graph::{RenderGraphExt, ViewNodeRunner},
		render_resource::PipelineCache,
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
		prefab::AddPrefabObserver,
		register_derived_component::RegisterDerivedComponent,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use components::{
	camera_labels::{CompositePass, SceneCamera, UiPass, WorldPass},
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
		TSavegame::register_savable_component::<AgentsPass>(app);
		TSavegame::register_savable_component::<OutlinePass>(app);
		TSavegame::register_savable_component::<CompositePass>(app);
		TSavegame::register_savable_component::<UiPass>(app);

		app.insert_resource(GlobalAmbientLight::NONE)
			.init_resource::<WindowSize>()
			.add_plugins((
				ExtractComponentPlugin::<PostProcessCamera>::default(),
				UniformComponentPlugin::<PostProcessCamera>::default(),
			))
			.add_plugins((
				ExtractComponentPlugin::<WorldPass>::default(),
				ExtractResourcePlugin::<DepthTexture<WorldPass>>::default(),
			))
			.add_plugins(ExtractResourcePlugin::<CameraRenderTarget<VisibilityPass>>::default())
			.add_plugins((
				ExtractComponentPlugin::<AgentsPass>::default(),
				ExtractResourcePlugin::<CameraRenderTarget<AgentsPass>>::default(),
				ExtractResourcePlugin::<DepthTexture<AgentsPass>>::default(),
			))
			.add_plugins((
				ExtractComponentPlugin::<OutlinePass>::default(),
				ExtractResourcePlugin::<CameraRenderTarget<OutlinePass>>::default(),
				ExtractResourcePlugin::<DepthTexture<OutlinePass>>::default(),
			))
			.register_required_components_with::<UiPass, TDebugCam>(self.debug_cam)
			.add_prefab_observer::<Player, ()>()
			.add_prefab_observer::<Enemy, ()>()
			.add_observer(WorldPass::insert_render_target)
			.add_observer(AgentsPass::insert_render_target)
			.add_observer(VisibilityPass::insert_render_target)
			.add_observer(OutlinePass::insert_render_target)
			.add_systems(
				Startup,
				(
					CameraRenderTarget::<WorldPass>::instantiate,
					CameraRenderTarget::<AgentsPass>::instantiate,
					CameraRenderTarget::<VisibilityPass>::instantiate,
					CameraRenderTarget::<OutlinePass>::instantiate,
					DepthTexture::<WorldPass>::instantiate,
					DepthTexture::<AgentsPass>::instantiate,
					DepthTexture::<OutlinePass>::instantiate,
				),
			)
			.add_systems(PostStartup, spawn_cameras)
			.add_systems(
				First,
				(
					WindowSize::update,
					CameraRenderTarget::<WorldPass>::update_size,
					CameraRenderTarget::<AgentsPass>::update_size,
					CameraRenderTarget::<VisibilityPass>::update_size,
					CameraRenderTarget::<OutlinePass>::update_size,
					DepthTexture::<WorldPass>::update_size,
					DepthTexture::<AgentsPass>::update_size,
					DepthTexture::<OutlinePass>::update_size,
				)
					.chain(),
			);

		let render_app = app.sub_app_mut(RenderApp);

		render_app
			.add_render_graph_node::<ViewNodeRunner<PostProcessNode>>(Core3d, PostProcessLabel)
			.add_render_graph_edges(Core3d, PostProcessLabel::EDGES)
			.add_systems(RenderStartup, PostProcessPipeline::init);

		render_app
			.add_render_graph_node::<ViewNodeRunner<CopyDepthTextureNode<WorldPass>>>(
				Core3d,
				DepthTextureLabel::for_pass(WorldPass),
			)
			.add_render_graph_node::<ViewNodeRunner<CopyDepthTextureNode<AgentsPass>>>(
				Core3d,
				DepthTextureLabel::for_pass(AgentsPass),
			)
			.add_render_graph_node::<ViewNodeRunner<CopyDepthTextureNode<OutlinePass>>>(
				Core3d,
				DepthTextureLabel::for_pass(OutlinePass),
			)
			.add_render_graph_edges(Core3d, DepthTextureLabel::LABELS);
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
	type TUiCamera = UiPass;
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
	type TRolesMut = RolesParamMut<'static, 'static>;
}
