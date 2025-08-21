pub mod force;
pub mod gravity;
pub mod health_damage;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub enum EffectApplies {
	#[default]
	Always,
	Once,
}
