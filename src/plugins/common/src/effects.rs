pub mod deal_damage;
pub mod force_shield;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum EffectApplies {
	Once,
	OncePerTarget,
	Always,
}
