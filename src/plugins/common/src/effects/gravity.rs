use crate::{
	attributes::effect_target::EffectTarget,
	tools::UnitsPerSecond,
	traits::handles_physics::Effect,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Gravity {
	pub strength: UnitsPerSecond,
}

impl Effect for Gravity {
	type TTarget = EffectTarget<Gravity>;
}
