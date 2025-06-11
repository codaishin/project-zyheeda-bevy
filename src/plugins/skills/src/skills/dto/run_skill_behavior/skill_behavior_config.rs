pub(crate) mod shape;
pub(crate) mod start_behavior;

use crate::behaviors::{
	SkillBehaviorConfig,
	build_skill_shape::BuildSkillShape,
	spawn_on::SpawnOn,
	start_behavior::SkillBehavior,
};
use serde::{Deserialize, Serialize};
use shape::SkillShapeDto;
use start_behavior::SkillBehaviorDto;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct SkillBehaviorConfigDto {
	shape: SkillShapeDto,
	contact: Vec<SkillBehaviorDto>,
	projection: Vec<SkillBehaviorDto>,
	spawn_on: SpawnOn,
}

impl From<SkillBehaviorConfigDto> for SkillBehaviorConfig {
	fn from(value: SkillBehaviorConfigDto) -> Self {
		let contact = value.contact.into_iter().map(SkillBehavior::from);
		let projection = value.projection.into_iter().map(SkillBehavior::from);

		Self {
			shape: BuildSkillShape::from(value.shape),
			contact: contact.collect(),
			projection: projection.collect(),
			spawn_on: value.spawn_on,
		}
	}
}

impl From<SkillBehaviorConfig> for SkillBehaviorConfigDto {
	fn from(value: SkillBehaviorConfig) -> Self {
		let contact = value.contact.into_iter().map(SkillBehaviorDto::from);
		let projection = value.projection.into_iter().map(SkillBehaviorDto::from);

		Self {
			shape: SkillShapeDto::from(value.shape),
			contact: contact.collect(),
			projection: projection.collect(),
			spawn_on: value.spawn_on,
		}
	}
}
