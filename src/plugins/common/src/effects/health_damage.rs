use super::EffectApplies;
use crate::{attributes::health::Health, traits::handles_physics::Effect};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct HealthDamage(pub f32, pub EffectApplies);

impl HealthDamage {
	pub const fn once(amount: f32) -> Self {
		HealthDamage(amount, EffectApplies::Once)
	}

	pub const fn per_second(amount: f32) -> Self {
		HealthDamage(amount, EffectApplies::OncePerSecond)
	}
}

impl Effect for HealthDamage {
	type TTarget = Health;
}
