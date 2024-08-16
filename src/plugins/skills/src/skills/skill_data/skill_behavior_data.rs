pub(crate) mod spawn_behavior_data;
pub(crate) mod start_behavior_data;

use crate::{
	behaviors::{spawn_behavior::SpawnBehavior, start_behavior::StartBehavior, Behavior},
	skills::{SkillBehavior, SkillBehaviors},
};
use serde::{Deserialize, Serialize};
use spawn_behavior_data::SpawnBehaviorData;
use start_behavior_data::StartBehaviorData;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct BehaviorData {
	spawn: SpawnBehaviorData,
	start: Vec<StartBehaviorData>,
}

impl From<BehaviorData> for Behavior {
	fn from(value: BehaviorData) -> Self {
		Self::new()
			.with_spawn(SpawnBehavior::from(value.spawn))
			.with_start(value.start.into_iter().map(StartBehavior::from).collect())
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SkillBehaviorsData {
	contact: BehaviorData,
	projection: BehaviorData,
}

impl From<SkillBehaviorsData> for SkillBehaviors {
	fn from(value: SkillBehaviorsData) -> Self {
		Self {
			contact: Behavior::from(value.contact),
			projection: Behavior::from(value.projection),
		}
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum SkillBehaviorData {
	OnActive(SkillBehaviorsData),
	OnAim(SkillBehaviorsData),
}

impl From<SkillBehaviorData> for SkillBehavior {
	fn from(value: SkillBehaviorData) -> Self {
		match value {
			SkillBehaviorData::OnActive(v) => Self::OnActive(SkillBehaviors::from(v)),
			SkillBehaviorData::OnAim(v) => Self::OnAim(SkillBehaviors::from(v)),
		}
	}
}
