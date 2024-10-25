use crate::components::effect_shader::EffectShaders;
use bevy::prelude::*;

#[derive(Bundle, Default)]
pub struct EffectShadersBundle {
	effect_shaders: EffectShaders,
}
