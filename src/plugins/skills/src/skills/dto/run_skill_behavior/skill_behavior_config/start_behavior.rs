use crate::behaviors::start_behavior::{
	SkillBehavior,
	deal_damage::StartDealingDamage,
	force::StartForce,
	gravity::StartGravity,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) enum SkillBehaviorDto {
	Gravity(StartGravity),
	Damage(StartDealingDamage),
	Force(StartForce),
}

impl From<SkillBehaviorDto> for SkillBehavior {
	fn from(value: SkillBehaviorDto) -> Self {
		match value {
			SkillBehaviorDto::Gravity(v) => Self::Gravity(v),
			SkillBehaviorDto::Damage(v) => Self::Damage(v),
			SkillBehaviorDto::Force(v) => Self::Force(v),
		}
	}
}

impl From<SkillBehavior> for SkillBehaviorDto {
	fn from(value: SkillBehavior) -> Self {
		match value {
			SkillBehavior::Gravity(v) => Self::Gravity(v),
			SkillBehavior::Damage(v) => Self::Damage(v),
			SkillBehavior::Force(v) => Self::Force(v),
			#[cfg(test)]
			SkillBehavior::Fn(_) => panic!("FN CANNOT BE SERIALIZED"),
		}
	}
}
