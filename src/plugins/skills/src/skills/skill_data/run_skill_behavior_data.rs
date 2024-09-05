pub(crate) mod skill_behavior_config_data;

use crate::{
	behaviors::SkillBehaviorConfig,
	skills::{
		lifetime::{OnActiveLifetime, OnAimLifeTime},
		RunSkillBehavior,
	},
};
use common::tools::duration_data::DurationData;
use serde::{Deserialize, Serialize};
use skill_behavior_config_data::SkillBehaviorConfigData;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum RunSkillBehaviorData {
	OnActive(SkillBehaviorConfigData<OnActiveLifetime<DurationData>>),
	OnAim(SkillBehaviorConfigData<OnAimLifeTime>),
}

impl From<RunSkillBehaviorData> for RunSkillBehavior {
	fn from(value: RunSkillBehaviorData) -> Self {
		match value {
			RunSkillBehaviorData::OnActive(v) => Self::OnActive(SkillBehaviorConfig::from(v)),
			RunSkillBehaviorData::OnAim(v) => Self::OnAim(SkillBehaviorConfig::from(v)),
		}
	}
}
