pub mod deal_damage;
pub mod force;
pub mod gravity;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub enum EffectApplies {
	#[default]
	Always,
	Once,
}
