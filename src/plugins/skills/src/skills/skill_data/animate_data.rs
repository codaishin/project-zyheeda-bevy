use super::skill_animation_data::SkillAnimationData;
use crate::skills::{Animate, SkillAnimation};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum AnimateData {
	Ignore,
	None,
	Some(SkillAnimationData),
}

impl From<AnimateData> for Animate<SkillAnimation> {
	fn from(value: AnimateData) -> Self {
		match value {
			AnimateData::Ignore => Animate::Ignore,
			AnimateData::None => Animate::None,
			AnimateData::Some(animation) => Animate::Some(animation.into()),
		}
	}
}
