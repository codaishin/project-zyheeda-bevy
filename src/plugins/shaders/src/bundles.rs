use crate::components::{effect_shader::EffectShaders, shadows_manager::ShadowsManager};
use bevy::prelude::*;

#[derive(Bundle, Default)]
pub struct EffectShadersBundle {
	effect_shaders: EffectShaders,
	shadows_manager: ShadowsManager,
}
