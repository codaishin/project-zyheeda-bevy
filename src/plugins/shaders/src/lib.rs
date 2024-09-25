pub mod components;
pub mod systems;
pub mod traits;

use bevy::prelude::*;
use common::systems::{
	move_component::MoveInto,
	move_component_from_children::MoveFromChildrenInto,
	remove_component::Remove,
	remove_component_from_children::RemoveFromChildren,
};
use components::effect_shader::EffectShaders;
use systems::{
	add_child_effect_shader::add_child_effect_shader,
	add_effect_shader::add_effect_shader,
	instantiate_effect_shaders::EffectShadersController,
};
use traits::get_effect_material::GetEffectMaterial;

pub struct ShaderPlugin;

impl Plugin for ShaderPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			PostUpdate,
			(
				Handle::<StandardMaterial>::remove_from::<EffectShaders>,
				Handle::<StandardMaterial>::remove_from_children_of::<EffectShaders>,
				Handle::<Mesh>::move_into::<EffectShaders>,
				Handle::<Mesh>::move_from_children_into::<EffectShaders>,
				EffectShadersController::instantiate_shaders,
			),
		);
	}
}

pub trait RegisterEffectShader {
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
