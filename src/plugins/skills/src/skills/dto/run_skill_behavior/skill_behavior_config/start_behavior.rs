use crate::behaviors::start_behavior::{
	SkillBehavior,
	deal_damage::StartDealingDamage,
	force_shield::StartForceShield,
	gravity::StartGravity,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum SkillBehaviorDto {
	Gravity(StartGravity),
	Damage(StartDealingDamage),
	ForceShield(StartForceShield),
}

impl From<SkillBehaviorDto> for SkillBehavior {
	fn from(value: SkillBehaviorDto) -> Self {
		match value {
			SkillBehaviorDto::Gravity(v) => Self::Gravity(v),
			SkillBehaviorDto::Damage(v) => Self::Damage(v),
			SkillBehaviorDto::ForceShield(v) => Self::ForceShield(v),
		}
	}
}
