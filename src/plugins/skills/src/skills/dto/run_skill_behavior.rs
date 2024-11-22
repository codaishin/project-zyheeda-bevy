pub(crate) mod skill_behavior_config;

use crate::{
	behaviors::SkillBehaviorConfig,
	skills::{
		lifetime::{OnActiveLifetime, OnAimLifeTime},
		RunSkillBehavior,
	},
};
use common::dto::duration::DurationDto;
use serde::{Deserialize, Serialize};
use skill_behavior_config::SkillBehaviorConfigDto;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum RunSkillBehaviorDto {
	OnActive(SkillBehaviorConfigDto<OnActiveLifetime<DurationDto>>),
	OnAim(SkillBehaviorConfigDto<OnAimLifeTime>),
}

impl From<RunSkillBehaviorDto> for RunSkillBehavior {
	fn from(value: RunSkillBehaviorDto) -> Self {
		match value {
			RunSkillBehaviorDto::OnActive(v) => Self::OnActive(SkillBehaviorConfig::from(v)),
			RunSkillBehaviorDto::OnAim(v) => Self::OnAim(SkillBehaviorConfig::from(v)),
		}
	}
}
