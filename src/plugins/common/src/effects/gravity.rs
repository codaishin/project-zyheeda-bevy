use super::EffectApplies;
use crate::tools::UnitsPerSecond;

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct Gravity {
	pub strength: UnitsPerSecond,
	pub effect_applies: EffectApplies,
}
