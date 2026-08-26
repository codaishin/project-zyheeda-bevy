use crate::{
	attributes::effect_target::EffectTarget,
	tools::UnitsPerSecond,
	traits::handles_physics::PhysicalEffect,
};
use macros::serde_model;

#[serde_model]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Gravity {
	pub strength: UnitsPerSecond,
}

impl PhysicalEffect for Gravity {
	type TTarget = EffectTarget<Gravity>;
}
