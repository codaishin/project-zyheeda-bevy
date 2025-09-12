use crate::{attributes::effect_target::EffectTarget, traits::handles_physics::Effect};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Force;

impl Effect for Force {
	type TTarget = EffectTarget<Force>;
}
