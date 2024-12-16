pub mod components;
pub mod materials;

pub(crate) mod systems;
pub(crate) mod traits;

use bevy::{prelude::*, render::render_resource::AsBindGroup};
use common::{
	components::essence::Essence,
	effects::{deal_damage::DealDamage, force_shield::ForceShield, gravity::Gravity},
	systems::{
		asset_process_delta::asset_process_delta,
		insert_associated::{Configure, InsertAssociated, InsertOn},
		remove_components::Remove,
		track_components::TrackComponentInSelfAndChildren,
	},
	traits::{
		handles_effect_shading::{HandlesEffectShading, HandlesEffectShadingFor},
		prefab::RegisterPrefab,
	},
};
use components::{
	effect_shader::EffectShader,
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

pub struct ShadersPlugin<TPrefabs>(PhantomData<TPrefabs>);

impl<TPrefabs> ShadersPlugin<TPrefabs>
where
	TPrefabs: Plugin + RegisterPrefab,
{
	pub fn depends_on(_: &TPrefabs) -> Self {
		Self(PhantomData)
	}

	fn build_for_effect_shaders(app: &mut App) {
		TPrefabs::register_prefab::<EffectShader<DealDamage>>(app);

		app.register_custom_effect_shader::<EffectShader<ForceShield>>()
			.register_custom_effect_shader::<EffectShader<Gravity>>()
			.register_standard_effect_shader::<EffectShader<DealDamage>>()
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
					EffectShadersTarget::remove_from_self_and_children::<Handle<StandardMaterial>>,
					EffectShadersTarget::track_in_self_and_children::<Handle<Mesh>>().system(),
					instantiate_effect_shaders,
				),
			);
	}

	fn build_for_essence_material(app: &mut App) {
		type InsertOnMeshWithEssence = InsertOn<Essence, With<Handle<Mesh>>, Changed<Essence>>;
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

impl<TPrefabs> Plugin for ShadersPlugin<TPrefabs>
where
	TPrefabs: Plugin + RegisterPrefab,
{
	fn build(&self, app: &mut App) {
		Self::build_for_effect_shaders(app);
		Self::build_for_essence_material(app);
	}
}

impl<TEffect, TPrefabs> HandlesEffectShadingFor<TEffect> for ShadersPlugin<TPrefabs>
where
	EffectShader<TEffect>: GetEffectMaterial + Sync + Send + 'static,
{
	fn effect_shader(effect: TEffect) -> impl Bundle {
		EffectShader(effect)
	}
}

impl<TPrefabs> HandlesEffectShading for ShadersPlugin<TPrefabs> {
	fn effect_shader_target() -> impl Bundle {
		EffectShadersTarget::default()
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

trait RegisterCustomEffectShader {
	fn register_custom_effect_shader<TEffectShader>(&mut self) -> &mut Self
	where
		TEffectShader: Component + GetEffectMaterial,
		TEffectShader::TMaterial: ShadowsAwareMaterial,
		<TEffectShader::TMaterial as AsBindGroup>::Data: PartialEq + Eq + Hash + Clone;
}

impl RegisterCustomEffectShader for App {
	fn register_custom_effect_shader<TEffectShader>(&mut self) -> &mut Self
	where
		TEffectShader: Component + GetEffectMaterial,
		TEffectShader::TMaterial: ShadowsAwareMaterial + Asset,
		<TEffectShader::TMaterial as AsBindGroup>::Data: PartialEq + Eq + Hash + Clone,
	{
		self.register_shader::<TEffectShader::TMaterial>();
		self.add_systems(
			Update,
			(
				add_effect_shader::<TEffectShader>,
				add_child_effect_shader::<TEffectShader>,
			),
		)
	}
}

trait RegisterStandardEffectShader {
	fn register_standard_effect_shader<TEffectShader>(&mut self) -> &mut Self
	where
		TEffectShader: Component + GetEffectMaterial;
}

impl RegisterStandardEffectShader for App {
	fn register_standard_effect_shader<TEffectShader>(&mut self) -> &mut Self
	where
		TEffectShader: Component + GetEffectMaterial,
	{
		self.add_systems(
			Update,
			(
				add_effect_shader::<TEffectShader>,
				add_child_effect_shader::<TEffectShader>,
			),
		)
	}
}
