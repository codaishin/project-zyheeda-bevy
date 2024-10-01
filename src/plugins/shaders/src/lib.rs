pub mod components;
pub mod materials;
pub mod systems;
pub mod traits;

use bevy::prelude::*;
use common::systems::{
	asset_process_delta::asset_process_delta,
	remove_component::Remove,
	remove_component_from_children::RemoveFromChildren,
	track_component::TrackComponent,
	track_component_from_children::TrackComponentInChildren,
};
use components::effect_shader::EffectShaders;
use interactions::components::{force::Force, gravity::Gravity};
use materials::{force_material::ForceMaterial, gravity_material::GravityMaterial};
use systems::{
	add_child_effect_shader::add_child_effect_shader,
	add_effect_shader::add_effect_shader,
	instantiate_effect_shaders::instantiate_effect_shaders,
};
use traits::get_effect_material::GetEffectMaterial;

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
				Handle::<StandardMaterial>::remove_from::<EffectShaders>,
				Handle::<StandardMaterial>::remove_from_children_of::<EffectShaders>,
				Handle::<Mesh>::track_by::<EffectShaders>,
				Handle::<Mesh>::track_in_children_by::<EffectShaders>,
				instantiate_effect_shaders,
			),
		);
	}
}

trait RegisterEffectShader {
	fn register_effect_shader<TEffect: Component + GetEffectMaterial>(&mut self) -> &mut Self;
}

impl RegisterEffectShader for App {
	fn register_effect_shader<TEffect: Component + GetEffectMaterial>(&mut self) -> &mut Self {
		self.add_systems(
			Update,
			(
				add_effect_shader::<TEffect>,
				add_child_effect_shader::<TEffect>,
			),
		)
	}
}
