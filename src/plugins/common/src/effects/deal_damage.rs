use super::EffectApplies;
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct DealDamage(pub f32, pub EffectApplies);

impl DealDamage {
	pub fn once(amount: f32) -> Self {
		DealDamage(amount, EffectApplies::Once)
	}

	pub fn once_per_second(amount: f32) -> Self {
		DealDamage(amount, EffectApplies::Always)
	}
}
