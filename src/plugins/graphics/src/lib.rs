pub mod components;
pub mod materials;

pub(crate) mod resources;
pub(crate) mod systems;
pub(crate) mod traits;

use bevy::{
	prelude::*,
	render::{
		render_resource::{AsBindGroup, PipelineCache},
		view::RenderLayers,
		RenderApp,
	},
};
use common::{
	components::essence::Essence,
	effects::{deal_damage::DealDamage, force_shield::ForceShield, gravity::Gravity},
	labels::Labels,
	systems::{
		insert_associated::{Configure, InsertAssociated, InsertOn},
		remove_components::Remove,
		track_components::TrackComponentInSelfAndChildren,
	},
	traits::{
		handles_effect::{HandlesAllEffects, HandlesEffect},
		handles_graphics::{MainCamera, UiRenderLayer},
		handles_load_tracking::{AssetsProgress, HandlesLoadTracking, InSubApp},
		handles_player::{WithCamera, WithMainCamera},
		handles_skill_behaviors::HandlesSkillBehaviors,
		prefab::RegisterPrefab,
		thread_safe::ThreadSafe,
	},
	PlayerCameras,
};
use components::{
	camera_labels::{FirstPass, FirstPassTexture, SecondPass, Ui},
	effect_shaders::EffectShader,
	effect_shaders_target::EffectShadersTarget,
	material_override::MaterialOverride,
};
use materials::essence_material::EssenceMaterial;
use resources::first_pass_image::FirstPassImage;
use std::{hash::Hash, marker::PhantomData};
use systems::{
	add_child_effect_shader::add_child_effect_shader,
	add_effect_shader::add_effect_shader,
	instantiate_effect_shaders::instantiate_effect_shaders,
	no_waiting_pipelines::no_waiting_pipelines,
	spawn_cameras::spawn_cameras,
};
use traits::{
	get_effect_material::GetEffectMaterial,
	shadows_aware_material::ShadowsAwareMaterial,
};

pub struct GraphicsPlugin<TPrefabs, TLoading, TInteractions, TBehaviors>(
	PhantomData<(TPrefabs, TLoading, TInteractions, TBehaviors)>,
);

impl<TPrefabs, TLoading, TInteractions, TBehaviors>
	GraphicsPlugin<TPrefabs, TLoading, TInteractions, TBehaviors>
where
	TPrefabs: ThreadSafe + RegisterPrefab,
	TLoading: ThreadSafe + HandlesLoadTracking,
	TInteractions: ThreadSafe + HandlesAllEffects,
	TBehaviors: ThreadSafe + HandlesSkillBehaviors,
{
	#[allow(clippy::type_complexity)]
	pub fn depends_on<TPlayers>(
		_: &TPrefabs,
		_: &TLoading,
		_: &TInteractions,
		_: &TBehaviors,
		player_plugin: TPlayers,
	) -> (
		Self,
		PlayerCameras!(TPlayers, FirstPass, Ui, SecondPass, FirstPassTexture),
	)
	where
		TPlayers: WithMainCamera,
	{
		(
			Self(PhantomData),
			player_plugin
				.with_main_camera::<FirstPass>()
				.with_camera::<FirstPassTexture>()
				.with_camera::<SecondPass>()
				.with_camera::<Ui>(),
		)
	}

	fn track_render_pipeline_ready(app: &mut App) {
		TLoading::register_load_tracking::<PipelineCache, AssetsProgress>().in_sub_app(
			app,
			RenderApp,
			ExtractSchedule,
			no_waiting_pipelines,
		);
	}

	fn effect_shaders(app: &mut App) {
		TPrefabs::register_prefab::<EffectShader<DealDamage>>(app);

		register_custom_effect_shader::<TInteractions, ForceShield>(app);
		register_custom_effect_shader::<TInteractions, Gravity>(app);
		register_effect_shader::<TInteractions, DealDamage>(app);

		app.add_systems(
			Labels::PREFAB_INSTANTIATION.label(),
			(
				InsertOn::<TBehaviors::TSkillContact>::associated::<EffectShadersTarget>(
					Configure::LeaveAsIs,
				),
				InsertOn::<TBehaviors::TSkillProjection>::associated::<EffectShadersTarget>(
					Configure::LeaveAsIs,
				),
			),
		)
		.add_systems(
			PostUpdate,
			(
				EffectShadersTarget::remove_from_self_and_children::<
					MeshMaterial3d<StandardMaterial>,
				>,
				EffectShadersTarget::track_in_self_and_children::<Mesh3d>().system(),
				instantiate_effect_shaders,
			),
		);
	}

	fn essence_material(app: &mut App) {
		type InsertOnMeshWithEssence = InsertOn<Essence, With<Mesh3d>, Changed<Essence>>;
		let configure_material = Configure::Apply(MaterialOverride::configure);

		app.register_shader::<EssenceMaterial>().add_systems(
			Update,
			(
				InsertOnMeshWithEssence::associated::<MaterialOverride>(configure_material),
				MaterialOverride::apply_material_exclusivity,
			)
				.chain(),
		);
	}

	fn cameras(app: &mut App) {
		app.add_systems(PostStartup, FirstPassImage::instantiate.pipe(spawn_cameras));
	}
}

impl<TPrefabs, TLoading, TInteractions, TBehaviors> Plugin
	for GraphicsPlugin<TPrefabs, TLoading, TInteractions, TBehaviors>
where
	TPrefabs: ThreadSafe + RegisterPrefab,
	TLoading: ThreadSafe + HandlesLoadTracking,
	TInteractions: ThreadSafe + HandlesAllEffects,
	TBehaviors: ThreadSafe + HandlesSkillBehaviors,
{
	fn build(&self, app: &mut App) {
		Self::track_render_pipeline_ready(app);
		Self::effect_shaders(app);
		Self::essence_material(app);
		Self::cameras(app);
	}
}

trait RegisterShader {
	fn register_shader<TMaterial>(&mut self) -> &mut Self
	where
		TMaterial: ShadowsAwareMaterial,
		TMaterial::Data: PartialEq + Eq + Hash + Clone;
}

impl RegisterShader for App {
	fn register_shader<TMaterial>(&mut self) -> &mut Self
	where
		TMaterial: ShadowsAwareMaterial,
		TMaterial::Data: PartialEq + Eq + Hash + Clone,
	{
		if self.is_plugin_added::<MaterialPlugin<TMaterial>>() {
			return self;
		}

		self.add_plugins(MaterialPlugin::<TMaterial> {
			shadows_enabled: TMaterial::shadows_enabled(),
			..default()
		})
	}
}

fn register_custom_effect_shader<TInteractions, TEffect>(app: &mut App)
where
	TInteractions: HandlesEffect<TEffect> + 'static,
	TEffect: GetEffectMaterial + Sync + Send + 'static,
	TEffect::TMaterial: ShadowsAwareMaterial,
	<TEffect::TMaterial as AsBindGroup>::Data: PartialEq + Eq + Hash + Clone,
{
	app.register_shader::<TEffect::TMaterial>();
	register_effect_shader::<TInteractions, TEffect>(app);
}

fn register_effect_shader<TInteractions, TEffect>(app: &mut App)
where
	TInteractions: HandlesEffect<TEffect> + 'static,
	TEffect: GetEffectMaterial + Sync + Send + 'static,
{
	app.add_systems(
		Labels::PREFAB_INSTANTIATION.label(),
		InsertOn::<TInteractions::TEffectComponent>::associated::<EffectShader<TEffect>>(
			Configure::LeaveAsIs,
		),
	);
	app.add_systems(
		Update,
		(
			add_effect_shader::<TInteractions, TEffect>,
			add_child_effect_shader::<TInteractions, TEffect>,
		),
	);
}

impl<TPrefabs, TLoading, TInteractions, TBehaviors> UiRenderLayer
	for GraphicsPlugin<TPrefabs, TLoading, TInteractions, TBehaviors>
{
	fn ui_render_layer() -> RenderLayers {
		Ui::render_layers()
	}
}

impl<TPrefabs, TLoading, TInteractions, TBehaviors> MainCamera
	for GraphicsPlugin<TPrefabs, TLoading, TInteractions, TBehaviors>
{
	type TMainCamera = FirstPass;
}
