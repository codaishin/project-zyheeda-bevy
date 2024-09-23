pub mod components;

use bevy::prelude::*;
use common::systems::{remove::Remove, remove_from_children::RemoveFromChildren};
use components::effect_shader::EffectShader;

pub struct ShaderPlugin;

impl Plugin for ShaderPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			(
				Handle::<StandardMaterial>::remove_from::<EffectShader>,
				Handle::<StandardMaterial>::remove_from_children_of::<EffectShader>,
			),
		);
	}
}
