use crate::components::effect_shader::EffectShader;

pub(crate) trait InsertEffectShader {
	fn insert_effect_shader(&mut self, effect_shader: &EffectShader);
}
