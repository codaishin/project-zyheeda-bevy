use crate::{
	attributes::effect_target::EffectTarget,
	tools::UnitsPerSecond,
	traits::handles_physics::PhysicalEffect,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub struct Gravity {
	pub strength: UnitsPerSecond,
}

impl PhysicalEffect for Gravity {
	type TTarget = EffectTarget<Gravity>;
}
