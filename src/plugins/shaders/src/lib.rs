pub mod bundles;
pub mod materials;

pub(crate) mod components;
pub(crate) mod systems;
pub(crate) mod traits;

use bevy::{prelude::*, render::render_resource::AsBindGroup};
use common::systems::{
	asset_process_delta::asset_process_delta,
	remove_components::Remove,
	track_components::TrackComponentInSelfAndChildren,
};
use components::effect_shader::EffectShaders;
use interactions::components::{force::Force, gravity::Gravity};
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

pub struct ShaderPlugin;

impl Plugin for ShaderPlugin {
	fn build(&self, app: &mut App) {
		app.register_shader::<EssenceMaterial>()
			.register_effect_shader_for::<Force>()
			.register_effect_shader_for::<Gravity>()
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
					EffectShaders::remove_from_self_and_children::<Handle<StandardMaterial>>,
					EffectShaders::track_in_self_and_children::<Handle<Mesh>>().system(),
					instantiate_effect_shaders,
				),
			);
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
	fn register_effect_shader_for<TEffect>(&mut self) -> &mut Self
	where
		TEffect: Component + GetEffectMaterial,
		TEffect::TMaterial: ShadowsAwareMaterial,
		<TEffect::TMaterial as AsBindGroup>::Data: PartialEq + Eq + Hash + Clone;
}

impl RegisterEffectShader for App {
	fn register_effect_shader_for<TEffect>(&mut self) -> &mut Self
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
