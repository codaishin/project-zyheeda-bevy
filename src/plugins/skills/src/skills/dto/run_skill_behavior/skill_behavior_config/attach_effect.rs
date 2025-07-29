use crate::behaviors::attach_skill_effect::{
	AttachEffect,
	deal_damage::AttachDealingDamage,
	force::AttachForce,
	gravity::AttachGravity,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) enum AttachEffectDto {
	Gravity(AttachGravity),
	Damage(AttachDealingDamage),
	Force(AttachForce),
}

impl From<AttachEffectDto> for AttachEffect {
	fn from(value: AttachEffectDto) -> Self {
		match value {
			AttachEffectDto::Gravity(v) => Self::Gravity(v),
			AttachEffectDto::Damage(v) => Self::Damage(v),
			AttachEffectDto::Force(v) => Self::Force(v),
		}
	}
}

impl From<AttachEffect> for AttachEffectDto {
	fn from(value: AttachEffect) -> Self {
		match value {
			AttachEffect::Gravity(v) => Self::Gravity(v),
			AttachEffect::Damage(v) => Self::Damage(v),
			AttachEffect::Force(v) => Self::Force(v),
			#[cfg(test)]
			AttachEffect::Fn(_) => panic!("FN CANNOT BE SERIALIZED"),
		}
	}
}
