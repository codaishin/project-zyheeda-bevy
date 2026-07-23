mod components;
mod materials;
mod observers;
mod resources;
mod system_params;
mod systems;
mod traits;

use crate::{
	components::{
		camera_labels::{AgentsPass, OutlinePass, WorldLight},
		model_render_layers::ModelRenderLayers,
		only_depth_prepass::OnlyDepthPrepass,
		post_process_camera::PostProcessCamera,
		roles::{Enemy, Player},
	},
	materials::{effect_material::EffectMaterial, lit_material::StandardLitMaterial},
	resources::{
		camera_parameters::CameraParameters,
		depth_texture::{CopyDepthTexture, DepthTexture},
		post_process_pipeline::SetupPostProcessPipeline,
	},
	system_params::{
		camera::{CameraParam, CameraParamMut},
		highlight::{HighlightParam, HighlightParamMut},
		lights::RolesParamMut,
	},
};
use bevy::{
	prelude::*,
	render::{RenderApp, render_resource::PipelineCache},
};
use common::{
	components::essence::Essence,
	effects::{force::Force, gravity::Gravity, health_damage::HealthDamage},
	states::game_state::LoadingGame,
	systems::log::OnError,
	tools::plugin_system_set::PluginSystemSet,
	traits::{
		after_plugin::AfterPlugin,
		handles_graphics::{HandlesCameras, HandlesGraphics},
		handles_load_tracking::{AssetsProgress, HandlesLoadTracking, LoadTrackingInSubApp},
		handles_physics::{HandlesAllPhysicalEffects, HandlesRaycast},
		handles_saving::HandlesSaving,
		handles_skill_physics::HandlesSkillPhysics,
		prefab::AddPrefabObserver,
		register_derived_component::RegisterDerivedComponent,
		system_set_definition::SystemSetDefinition,
		thread_safe::ThreadSafe,
	},
};
use components::{
	camera_labels::{UiPass, WorldPass},
	effect_material_handle::EffectMaterialHandle,
	material_override::MaterialOverride,
};
use materials::essence_material::EssenceMaterial;
use resources::window_size::WindowSize;
use std::{hash::Hash, marker::PhantomData};
use systems::no_waiting_pipelines::no_waiting_pipelines;

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
	TPhysics: ThreadSafe
		+ SystemSetDefinition
		+ HandlesRaycast
		+ HandlesAllPhysicalEffects
		+ HandlesSkillPhysics,
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
	TPhysics: ThreadSafe
		+ SystemSetDefinition
		+ HandlesRaycast
		+ HandlesAllPhysicalEffects
		+ HandlesSkillPhysics,
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
			.add_plugins(MaterialPlugin::<StandardLitMaterial>::default())
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
					.in_set(GraphicSystems)
					.after_plugin(TPhysics::SYSTEMS),
			);
	}

	fn cameras(&self, app: &mut App) {
		TSavegame::register_savable_component::<UiPass>(app);

		app.insert_resource(GlobalAmbientLight::NONE)
			.init_resource::<WindowSize>()
			.init_resource::<CameraParameters>()
			.register_required_components_with::<UiPass, TDebugCam>(self.debug_cam)
			.copy_depth_texture::<WorldPass>()
			.copy_depth_texture::<AgentsPass>()
			.copy_depth_texture::<OutlinePass>()
			.setup_post_process_pipeline()
			.add_prefab_observer::<Player, ()>()
			.add_prefab_observer::<Enemy, ()>()
			.add_prefab_observer::<WorldLight, ()>()
			.add_systems(PostStartup, UiPass::spawn)
			.add_systems(
				Update,
				(
					UiPass::process_new_ui_pass.pipe(OnError::log),
					WorldPass::update_target_ray::<TPhysics::TRaycastMut>.pipe(OnError::log),
					CameraParameters::apply_changes,
				)
					.chain()
					.in_set(GraphicSystems)
					.after_plugin(TPhysics::SYSTEMS),
			)
			.add_systems(
				First,
				(WindowSize::update, OnlyDepthPrepass::update_render_targets).chain(),
			);
	}
}

impl<TDebugCam, TLoading, TSavegame, TPhysics> Plugin
	for GraphicsPlugin<TDebugCam, (TLoading, TSavegame, TPhysics)>
where
	TDebugCam: Component,
	TLoading: ThreadSafe + HandlesLoadTracking,
	TSavegame: ThreadSafe + HandlesSaving,
	TPhysics: ThreadSafe
		+ SystemSetDefinition
		+ HandlesRaycast
		+ HandlesAllPhysicalEffects
		+ HandlesSkillPhysics,
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

impl<TDebugCam, TDependencies> HandlesCameras for GraphicsPlugin<TDebugCam, TDependencies> {
	type TCamera = CameraParam<'static>;
	type TCameraMut = CameraParamMut<'static, 'static>;
}

impl<TDebugCam, TDependencies> HandlesGraphics for GraphicsPlugin<TDebugCam, TDependencies> {
	type THighlight = HighlightParam<'static, 'static>;
	type THighlightMut = HighlightParamMut<'static, 'static>;
	type TRolesMut = RolesParamMut<'static, 'static>;
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone)]
pub struct GraphicSystems;

impl<TDebugCam, TDependencies> SystemSetDefinition for GraphicsPlugin<TDebugCam, TDependencies> {
	type TSystemSet = GraphicSystems;

	const SYSTEMS: PluginSystemSet<GraphicSystems> = PluginSystemSet::from_set(GraphicSystems);
}
