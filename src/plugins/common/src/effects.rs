pub mod deal_damage;
pub mod force_shield;
pub mod gravity;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub enum EffectApplies {
	#[default]
	Always,
	Once,
	OncePerTarget,
}
