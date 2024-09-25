use crate::components::effect_shader::EffectShader;

pub(crate) trait InsertUnmovableEffectShader {
	fn insert_unmovable_effect_shader(&mut self, effect_shader: &EffectShader);
}
