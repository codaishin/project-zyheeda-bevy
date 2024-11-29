use bevy::prelude::*;

pub trait HandlesEffectShading {
	fn effect_shader_target() -> impl Bundle;
}
