pub mod components;

use bevy::prelude::*;
use common::systems::remove::remove_from;
use components::effect_shader::EffectShader;

pub struct ShaderPlugin;

impl Plugin for ShaderPlugin {
	fn build(&self, app: &mut App) {
		app.add_systems(
			Update,
			remove_from::<EffectShader, Handle<StandardMaterial>>,
		);
	}
}
