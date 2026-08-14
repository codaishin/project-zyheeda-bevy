use crate::{attributes::effect_target::EffectTarget, traits::handles_physics::PhysicalEffect};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Force;

impl PhysicalEffect for Force {
	type TTarget = EffectTarget<Force>;
}
