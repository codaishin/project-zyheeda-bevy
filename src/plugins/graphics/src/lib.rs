pub mod components;
pub mod materials;

pub(crate) mod resources;
pub(crate) mod systems;
pub(crate) mod traits;

use bevy::{
	prelude::*,
	render::{
		RenderApp,
		render_resource::{AsBindGroup, PipelineCache},
	},
};
use common::{
	components::essence::Essence,
	effects::{deal_damage::DealDamage, force_shield::ForceShield, gravity::Gravity},
	systems::{
		insert_required::{InsertOn, InsertRequired},
		remove_components::Remove,
		track_components::TrackComponentInSelfAndChildren,
	},
	traits::{
		handles_effect::{HandlesAllEffects, HandlesEffect},
		handles_graphics::{FirstPassCamera, UiCamera, WorldCameras},
		handles_load_tracking::{AssetsProgress, HandlesLoadTracking, InSubApp},
		handles_skill_behaviors::HandlesSkillBehaviors,
		prefab::RegisterPrefab,
		thread_safe::ThreadSafe,
	},
};
use components::{
	camera_labels::{FirstPass, PlayerCamera, SecondPass, Ui},
	effect_shaders::{EffectShader, damage_effect_shaders::DamageEffectShaders},
	effect_shaders_target::EffectShadersTarget,
	material_override::MaterialOverride,
};
use materials::essence_material::EssenceMaterial;
use resources::{first_pass_image::FirstPassImage, window_size::WindowSize};
use std::{hash::Hash, marker::PhantomData};
use systems::{
	add_child_effect_shader::add_child_effect_shader,
	add_effect_shader::add_effect_shader,
	insert_effect_shader_render_layers::insert_effect_shader_render_layers,
	instantiate_effect_shaders::instantiate_effect_shaders,
	no_waiting_pipelines::no_waiting_pipelines,
	spawn_cameras::spawn_cameras,
};
use traits::{
	get_effect_material::GetEffectMaterial,
	shadows_aware_material::ShadowsAwareMaterial,
};

pub struct GraphicsPlugin<TDependencies>(PhantomData<TDependencies>);

impl<TPrefabs, TLoading, TInteractions, TBehaviors>
	GraphicsPlugin<(TPrefabs, TLoading, TInteractions, TBehaviors)>
where
	TPrefabs: ThreadSafe + RegisterPrefab,
	TLoading: ThreadSafe + HandlesLoadTracking,
	TInteractions: ThreadSafe + HandlesAllEffects,
	TBehaviors: ThreadSafe + HandlesSkillBehaviors,
{
	#[allow(clippy::type_complexity)]
	pub fn depends_on(_: &TPrefabs, _: &TLoading, _: &TInteractions, _: &TBehaviors) -> Self {
		Self(PhantomData)
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
		register_custom_effect_shader::<TInteractions, ForceShield>(app);
		register_custom_effect_shader::<TInteractions, Gravity>(app);
		register_effect_shader::<TInteractions, DealDamage>(app);

		app.register_required_components::<TBehaviors::TSkillContact, EffectShadersTarget>()
			.register_required_components::<TBehaviors::TSkillProjection, EffectShadersTarget>()
			.register_required_components::<EffectShader<DealDamage>, DamageEffectShaders>();
		app.add_systems(
			PostUpdate,
			(
				EffectShadersTarget::remove_from_self_and_children::<
					MeshMaterial3d<StandardMaterial>,
				>,
				EffectShadersTarget::track_in_self_and_children::<Mesh3d>().system(),
				instantiate_effect_shaders,
				insert_effect_shader_render_layers(SecondPass),
			)
				.chain(),
		);
	}

	fn essence_material(app: &mut App) {
		type InsertOnMeshWithEssence = InsertOn<Essence, With<Mesh3d>, Changed<Essence>>;

		app.register_shader::<EssenceMaterial>().add_systems(
			Update,
			(
				InsertOnMeshWithEssence::required::<MaterialOverride>(|essence| {
					MaterialOverride::from(essence)
				}),
				MaterialOverride::apply_material_exclusivity,
			)
				.chain(),
		);
	}

	fn cameras(app: &mut App) {
		app.init_resource::<WindowSize>()
			.add_systems(PostStartup, FirstPassImage::instantiate.pipe(spawn_cameras))
			.add_systems(
				First,
				(WindowSize::update, FirstPassImage::<Image>::update_size).chain(),
			);
	}
}

impl<TPrefabs, TLoading, TInteractions, TBehaviors> Plugin
	for GraphicsPlugin<(TPrefabs, TLoading, TInteractions, TBehaviors)>
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
	app.register_required_components::<TInteractions::TEffectComponent, EffectShader<TEffect>>();
	app.add_systems(
		Update,
		(
			add_effect_shader::<TInteractions, TEffect>,
			add_child_effect_shader::<TInteractions, TEffect>,
		),
	);
}

impl<TDependencies> UiCamera for GraphicsPlugin<TDependencies> {
	type TUiCamera = Ui;
}

impl<TDependencies> FirstPassCamera for GraphicsPlugin<TDependencies> {
	type TFirstPassCamera = FirstPass;
}

impl<TDependencies> WorldCameras for GraphicsPlugin<TDependencies> {
	type TWorldCameras = PlayerCamera;
}
