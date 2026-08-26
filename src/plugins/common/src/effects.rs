pub mod force;
pub mod gravity;
pub mod health_damage;

use macros::serde_model;

#[serde_model]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum EffectApplies {
	#[default]
	OncePerSecond,
	Once,
}
