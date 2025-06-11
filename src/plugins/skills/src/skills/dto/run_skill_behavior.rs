pub(crate) mod skill_behavior_config;

use crate::{behaviors::SkillBehaviorConfig, skills::RunSkillBehavior};
use serde::{Deserialize, Serialize};
use skill_behavior_config::SkillBehaviorConfigDto;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) enum RunSkillBehaviorDto {
	OnActive(SkillBehaviorConfigDto),
	OnAim(SkillBehaviorConfigDto),
}

impl From<RunSkillBehaviorDto> for RunSkillBehavior {
	fn from(value: RunSkillBehaviorDto) -> Self {
		match value {
			RunSkillBehaviorDto::OnActive(v) => Self::OnActive(SkillBehaviorConfig::from(v)),
			RunSkillBehaviorDto::OnAim(v) => Self::OnAim(SkillBehaviorConfig::from(v)),
		}
	}
}

impl From<RunSkillBehavior> for RunSkillBehaviorDto {
	fn from(value: RunSkillBehavior) -> Self {
		match value {
			RunSkillBehavior::OnActive(v) => Self::OnActive(SkillBehaviorConfigDto::from(v)),
			RunSkillBehavior::OnAim(v) => Self::OnAim(SkillBehaviorConfigDto::from(v)),
		}
	}
}
