pub mod bundles;

pub(crate) mod components;
pub(crate) mod materials;
pub(crate) mod systems;
pub(crate) mod traits;

use bevy::prelude::*;
use common::systems::{
	asset_process_delta::asset_process_delta,
	remove_components::Remove,
	track_components::TrackComponentInSelfAndChildren,
};
use components::{effect_shader::EffectShaders, shadows_manager::ShadowsManager};
use interactions::components::{force::Force, gravity::Gravity};
use materials::{force_material::ForceMaterial, gravity_material::GravityMaterial};
use systems::{
	add_child_effect_shader::add_child_effect_shader,
	add_effect_shader::add_effect_shader,
	instantiate_effect_shaders::instantiate_effect_shaders,
};
use traits::{effect_material::EffectMaterial, get_effect_material::GetEffectMaterial};

pub struct ShaderPlugin;

impl Plugin for ShaderPlugin {
	fn build(&self, app: &mut App) {
		app.add_plugins((
			MaterialPlugin::<ForceMaterial>::default(),
			MaterialPlugin::<GravityMaterial>::default(),
		))
		.register_effect_shader::<Force>()
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
				EffectShaders::remove_from_self_and_children::<Handle<StandardMaterial>>,
				EffectShaders::track_in_self_and_children::<Handle<Mesh>>().system(),
				instantiate_effect_shaders,
				ShadowsManager::system,
			),
		);
	}
}

trait RegisterEffectShader {
	fn register_effect_shader<TEffect>(&mut self) -> &mut Self
	where
		TEffect: Component + GetEffectMaterial,
		TEffect::TMaterial: EffectMaterial;
}

impl RegisterEffectShader for App {
	fn register_effect_shader<TEffect>(&mut self) -> &mut Self
	where
		TEffect: Component + GetEffectMaterial,
		TEffect::TMaterial: EffectMaterial + Asset,
	{
		self.add_systems(
			Update,
			ShadowsManager::track_in_self_and_children::<Handle<TEffect::TMaterial>>().system(),
		);
		self.add_systems(
			Update,
			(
				add_effect_shader::<TEffect>,
				add_child_effect_shader::<TEffect>,
			),
		)
	}
}
