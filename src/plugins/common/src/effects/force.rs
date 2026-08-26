use crate::{attributes::effect_target::EffectTarget, traits::handles_physics::PhysicalEffect};
use macros::serde_model;

#[serde_model]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Force;

impl PhysicalEffect for Force {
	type TTarget = EffectTarget<Force>;
}
