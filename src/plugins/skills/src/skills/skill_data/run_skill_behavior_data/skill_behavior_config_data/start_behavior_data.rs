use crate::behaviors::start_behavior::{
	force::StartForce,
	start_deal_damage::StartDealingDamage,
	start_gravity::StartGravity,
	SkillBehavior,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum SkillBehaviorData {
	Gravity(StartGravity),
	Damage(StartDealingDamage),
	Force(StartForce),
}

impl From<SkillBehaviorData> for SkillBehavior {
	fn from(value: SkillBehaviorData) -> Self {
		match value {
			SkillBehaviorData::Gravity(v) => Self::Gravity(v),
			SkillBehaviorData::Damage(v) => Self::Damage(v),
			SkillBehaviorData::Force(v) => Self::Force(v),
		}
	}
}
