pub mod components;
pub mod materials;

pub(crate) mod systems;
pub(crate) mod traits;

use bevy::{prelude::*, render::render_resource::AsBindGroup};
use common::{
	components::essence::Essence,
	effects::{deal_damage::DealDamage, force_shield::ForceShield, gravity::Gravity},
	labels::Labels,
	systems::{
		asset_process_delta::asset_process_delta,
		insert_associated::{Configure, InsertAssociated, InsertOn},
		remove_components::Remove,
		track_components::TrackComponentInSelfAndChildren,
	},
	traits::{
		handles_effect::{HandlesAllEffects, HandlesEffect},
		handles_skill_behaviors::HandlesSkillBehaviors,
		prefab::RegisterPrefab,
	},
};
use components::{
	effect_shaders::EffectShader,
	effect_shaders_target::EffectShadersTarget,
	material_override::MaterialOverride,
};
use materials::{
	essence_material::EssenceMaterial,
	force_material::ForceMaterial,
	gravity_material::GravityMaterial,
};
use std::{hash::Hash, marker::PhantomData};
use systems::{
	add_child_effect_shader::add_child_effect_shader,
	add_effect_shader::add_effect_shader,
	instantiate_effect_shaders::instantiate_effect_shaders,
};
use traits::{
	get_effect_material::GetEffectMaterial,
	shadows_aware_material::ShadowsAwareMaterial,
};

pub struct ShadersPlugin<TPrefabs, TInteractions, TBehaviors>(
	PhantomData<(TPrefabs, TInteractions, TBehaviors)>,
);

impl<TPrefabs, TInteractions, TBehaviors> ShadersPlugin<TPrefabs, TInteractions, TBehaviors>
where
	TPrefabs: Plugin + RegisterPrefab,
	TInteractions: Plugin + HandlesAllEffects,
	TBehaviors: Plugin + HandlesSkillBehaviors,
{
	pub fn depends_on(_: &TPrefabs, _: &TInteractions, _: &TBehaviors) -> Self {
		Self(PhantomData)
	}

	fn build_for_effect_shaders(app: &mut App) {
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
			Update,
			(
				asset_process_delta::<ForceMaterial, Virtual>,
				asset_process_delta::<GravityMaterial, Virtual>,
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

	fn build_for_essence_material(app: &mut App) {
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
}

impl<TPrefabs, TInteractions, TBehaviors> Plugin
	for ShadersPlugin<TPrefabs, TInteractions, TBehaviors>
where
	TPrefabs: Plugin + RegisterPrefab,
	TInteractions: Plugin + HandlesAllEffects,
	TBehaviors: Plugin + HandlesSkillBehaviors,
{
	fn build(&self, app: &mut App) {
		Self::build_for_effect_shaders(app);
		Self::build_for_essence_material(app);
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
