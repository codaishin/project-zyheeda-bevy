use crate::{
	skills::{shoot_hand_gun::ShootHandGun, SkillAnimation},
	traits::GetSkillAnimation,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum SkillAnimationData {
	ShootHandGun,
}

impl From<SkillAnimationData> for SkillAnimation {
	fn from(value: SkillAnimationData) -> Self {
		match value {
			SkillAnimationData::ShootHandGun => ShootHandGun::animation(),
		}
	}
}
