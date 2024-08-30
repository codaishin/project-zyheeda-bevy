pub(crate) mod spawn_behavior_data;
pub(crate) mod start_behavior_data;

use crate::{
	behaviors::{spawn_behavior::SkillShape, start_behavior::SkillBehavior, SkillBehaviorConfig},
	skills::RunSkillBehavior,
};
use serde::{Deserialize, Serialize};
use spawn_behavior_data::SkillShapeData;
use start_behavior_data::SkillBehaviorData;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SkillBehaviorConfigData {
	shape: SkillShapeData,
	contact: Vec<SkillBehaviorData>,
	projection: Vec<SkillBehaviorData>,
}

impl From<SkillBehaviorConfigData> for SkillBehaviorConfig {
	fn from(value: SkillBehaviorConfigData) -> Self {
		let shape = SkillShape::from(value.shape);
		let contact = value.contact.into_iter().map(SkillBehavior::from);
		let projection = value.projection.into_iter().map(SkillBehavior::from);
		Self::new()
			.with_shape(shape)
			.with_contact_behaviors(contact.collect())
			.with_projection_behaviors(projection.collect())
	}
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) enum RunSkillBehaviorData {
	OnActive(SkillBehaviorConfigData),
	OnAim(SkillBehaviorConfigData),
}

impl From<RunSkillBehaviorData> for RunSkillBehavior {
	fn from(value: RunSkillBehaviorData) -> Self {
		match value {
			RunSkillBehaviorData::OnActive(v) => Self::OnActive(SkillBehaviorConfig::from(v)),
			RunSkillBehaviorData::OnAim(v) => Self::OnAim(SkillBehaviorConfig::from(v)),
		}
	}
}
