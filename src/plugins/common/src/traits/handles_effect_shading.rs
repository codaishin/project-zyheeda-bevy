use crate::effects::force_shield::ForceShield;
use bevy::prelude::*;

pub trait HandlesEffectShadingFor<TEffect> {
	fn effect_shader(effect: TEffect) -> impl Bundle;
}

pub trait HandlesEffectShading: HandlesEffectShadingFor<ForceShield> {
	fn effect_shader_target() -> impl Bundle;
}
