pub mod components;

use bevy::prelude::*;
use common::systems::{remove::remove_from, remove_from_children::remove_from_children_of};
use components::effect_shader::EffectShader;

pub struct ShaderPlugin;

impl Plugin for ShaderPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			(
				remove_from::<EffectShader, Handle<StandardMaterial>>,
				remove_from_children_of::<EffectShader, Handle<StandardMaterial>>,
			),
		);
	}
}
