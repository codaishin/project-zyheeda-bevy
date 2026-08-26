use super::EffectApplies;
use crate::{attributes::health::Health, traits::handles_physics::PhysicalEffect};
use macros::serde_model;

#[serde_model]
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct HealthDamage(pub f32, pub EffectApplies);

impl HealthDamage {
	pub const fn once(amount: f32) -> Self {
		HealthDamage(amount, EffectApplies::Once)
	}

	pub const fn per_second(amount: f32) -> Self {
		HealthDamage(amount, EffectApplies::OncePerSecond)
	}
}

impl PhysicalEffect for HealthDamage {
	type TTarget = Health;
}
