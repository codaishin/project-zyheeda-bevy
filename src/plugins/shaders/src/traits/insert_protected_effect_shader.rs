use crate::components::effect_shaders_target::EffectShaderHandle;

pub(crate) trait InsertProtectedEffectShader {
	fn insert_protected_effect_shader(&mut self, effect_shader: &EffectShaderHandle);
}
