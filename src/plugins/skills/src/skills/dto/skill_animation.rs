use crate::{
	skills::{shoot_hand_gun::ShootHandGun, SkillAnimation},
	traits::GetSkillAnimation,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum SkillAnimationDto {
	ShootHandGun,
}

impl From<SkillAnimationDto> for SkillAnimation {
	fn from(value: SkillAnimationDto) -> Self {
		match value {
			SkillAnimationDto::ShootHandGun => ShootHandGun::animation(),
		}
	}
}
