pub mod components;
pub mod materials;

pub(crate) mod systems;
pub(crate) mod traits;

use bevy::{prelude::*, render::render_resource::AsBindGroup};
use common::{
	components::essence::Essence,
	effects::force_shield::ForceShield,
	systems::{
		asset_process_delta::asset_process_delta,
		insert_associated::{Configure, InsertAssociated, InsertOn},
		remove_components::Remove,
		track_components::TrackComponentInSelfAndChildren,
	},
	traits::handles_effect_shading::{HandlesEffectShading, HandlesEffectShadingFor},
};
use components::{
	effect_shader::EffectShader,
	effect_shaders_target::EffectShadersTarget,
	material_override::MaterialOverride,
};
use interactions::components::gravity::Gravity;
use materials::{
	essence_material::EssenceMaterial,
	force_material::ForceMaterial,
	gravity_material::GravityMaterial,
};
use std::hash::Hash;
use systems::{
	add_child_effect_shader::add_child_effect_shader,
	add_effect_shader::add_effect_shader,
	instantiate_effect_shaders::instantiate_effect_shaders,
};
use traits::{
	get_effect_material::GetEffectMaterial,
	shadows_aware_material::ShadowsAwareMaterial,
};

pub struct ShadersPlugin;

impl ShadersPlugin {
	fn build_for_effect_shaders(app: &mut App) {
		app.register_effect_shader::<EffectShader<ForceShield>>()
			.register_effect_shader::<Gravity>()
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

impl Plugin for ShadersPlugin {
	fn build(&self, app: &mut App) {
		Self::build_for_effect_shaders(app);
		Self::build_for_essence_material(app);
	}
}

trait ShadedEffect {}

impl ShadedEffect for ForceShield {}

impl<TEffect> HandlesEffectShadingFor<TEffect> for ShadersPlugin
where
	TEffect: ShadedEffect + Sync + Send + 'static,
{
	fn effect_shader(effect: TEffect) -> impl Bundle {
		EffectShader(effect)
	}
}

impl HandlesEffectShading for ShadersPlugin {
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

trait RegisterEffectShader {
	fn register_effect_shader<TEffect>(&mut self) -> &mut Self
	where
		TEffect: Component + GetEffectMaterial,
		TEffect::TMaterial: ShadowsAwareMaterial,
		<TEffect::TMaterial as AsBindGroup>::Data: PartialEq + Eq + Hash + Clone;
}

impl RegisterEffectShader for App {
	fn register_effect_shader<TEffect>(&mut self) -> &mut Self
	where
		TEffect: Component + GetEffectMaterial,
		TEffect::TMaterial: ShadowsAwareMaterial + Asset,
		<TEffect::TMaterial as AsBindGroup>::Data: PartialEq + Eq + Hash + Clone,
	{
		self.register_shader::<TEffect::TMaterial>();
		self.add_systems(
			Update,
			(
				add_effect_shader::<TEffect>,
				add_child_effect_shader::<TEffect>,
			),
		)
	}
}
