use crate::effects::{force_shield::ForceShield, gravity::Gravity};
use bevy::prelude::*;

pub trait HandlesEffectShadingFor<TEffect> {
	fn effect_shader(effect: TEffect) -> impl Bundle;
}

pub trait HandlesEffectShadingForAll:
	HandlesEffectShadingFor<ForceShield> + HandlesEffectShadingFor<Gravity>
{
}

impl<T> HandlesEffectShadingForAll for T where
	T: HandlesEffectShadingFor<ForceShield> + HandlesEffectShadingFor<Gravity>
{
}

pub trait HandlesEffectShading: HandlesEffectShadingForAll {
	fn effect_shader_target() -> impl Bundle;
}
