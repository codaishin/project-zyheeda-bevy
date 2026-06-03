mod components;
mod materials;
mod observers;
mod resources;
mod systems;
mod traits;

use crate::{
	components::effect_material_config::EffectShaderMeshOf,
	materials::effect_material::EffectMaterial,
};
use bevy::{
	prelude::*,
	render::{RenderApp, render_resource::PipelineCache},
};
use common::{
	components::essence::Essence,
	effects::{force::Force, gravity::Gravity, health_damage::HealthDamage},
	states::game_state::LoadingGame,
	systems::link::to_target::LinkToTarget,
	traits::{
		after_plugin::AfterPlugin,
		handles_graphics::{FirstPassCamera, UiCamera, WorldCameras},
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
	camera_labels::{FirstPass, SecondPass, Ui, WorldCamera},
	effect_material_config::EffectShader,
	material_override::MaterialOverride,
};
use materials::essence_material::EssenceMaterial;
use resources::{first_pass_image::FirstPassImage, window_size::WindowSize};
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

	fn effect_shading(app: &mut App) {
		type UnlinkedEffectShader = (Added<Mesh3d>, Without<EffectShaderMeshOf>);

		app.add_plugins(MaterialPlugin::<EffectMaterial>::default())
			.add_observer(EffectShader::add_to::<TPhysics::TSkillContact>)
			.add_observer(EffectShader::add_to::<TPhysics::TSkillProjection>)
			.add_systems(
				Update,
				(
					UnlinkedEffectShader::link_to::<EffectShader, EffectShaderMeshOf>,
					EffectShader::modify_material::<TPhysics, Force>,
					EffectShader::modify_material::<TPhysics, Gravity>,
					EffectShader::modify_material::<TPhysics, HealthDamage>,
					EffectShader::propagate(SecondPass),
				)
					.chain()
					.after_plugin(TPhysics::SYSTEMS),
			);
	}

	fn essence_material(app: &mut App) {
		app.register_derived_component::<Essence, MaterialOverride>()
			.register_shader::<EssenceMaterial>()
			.add_observer(MaterialOverride::update_essence_shader);
	}

	fn cameras(&self, app: &mut App) {
		app.register_required_components::<WorldCamera, TSavegame::TSaveEntityMarker>();
		TSavegame::register_savable_component::<FirstPass>(app);
		TSavegame::register_savable_component::<SecondPass>(app);
		TSavegame::register_savable_component::<Ui>(app);

		app.init_resource::<WindowSize>()
			.register_required_components_with::<SecondPass, TDebugCam>(self.debug_cam)
			.add_observer(FirstPass::insert_camera)
			.add_systems(Startup, FirstPassImage::instantiate)
			.add_systems(PostStartup, spawn_cameras)
			.add_systems(
				First,
				(WindowSize::update, FirstPassImage::<Image>::update_size).chain(),
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
		Self::effect_shading(app);
		Self::essence_material(app);
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
	type TFirstPassCamera = FirstPass;
}

impl<TDebugCam, TDependencies> WorldCameras for GraphicsPlugin<TDebugCam, TDependencies> {
	type TWorldCameras = WorldCamera;
}
