use super::EffectApplies;
use crate::{attributes::health::Health, traits::handles_effects::Effect};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct HealthDamage(pub f32, pub EffectApplies);

impl HealthDamage {
	pub fn once(amount: f32) -> Self {
		HealthDamage(amount, EffectApplies::Once)
	}

	pub fn once_per_second(amount: f32) -> Self {
		HealthDamage(amount, EffectApplies::Always)
	}
}

impl Effect for HealthDamage {
	type TTarget = Health;
}
