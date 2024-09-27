use crate::components::effect_shader::EffectShader;

pub(crate) trait RemoveUnmovableEffectShader {
	fn remove_unmovable_effect_shader(&mut self, effect_shader: &EffectShader);
}
