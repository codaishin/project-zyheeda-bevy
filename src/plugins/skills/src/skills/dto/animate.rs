use super::skill_animation::SkillAnimationDto;
use crate::skills::{Animate, SkillAnimation};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum AnimateDto {
	Ignore,
	None,
	Some(SkillAnimationDto),
}

impl From<AnimateDto> for Animate<SkillAnimation> {
	fn from(value: AnimateDto) -> Self {
		match value {
			AnimateDto::Ignore => Animate::Ignore,
			AnimateDto::None => Animate::None,
			AnimateDto::Some(animation) => Animate::Some(animation.into()),
		}
	}
}
