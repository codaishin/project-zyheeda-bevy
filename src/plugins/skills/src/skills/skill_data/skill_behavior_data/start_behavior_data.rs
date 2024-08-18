use crate::behaviors::start_behavior::{
	force::StartForce,
	start_deal_damage::StartDealingDamage,
	start_gravity::StartGravity,
	StartBehavior,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum StartBehaviorData {
	Gravity(StartGravity),
	Damage(StartDealingDamage),
	Force(StartForce),
}

impl From<StartBehaviorData> for StartBehavior {
	fn from(value: StartBehaviorData) -> Self {
		match value {
			StartBehaviorData::Gravity(v) => Self::Gravity(v),
			StartBehaviorData::Damage(v) => Self::Damage(v),
			StartBehaviorData::Force(v) => Self::Force(v),
		}
	}
}
