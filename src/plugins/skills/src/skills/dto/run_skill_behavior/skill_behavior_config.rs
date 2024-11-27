pub(crate) mod shape;
pub(crate) mod start_behavior;

use crate::behaviors::{
	build_skill_shape::BuildSkillShape,
	spawn_on::SpawnOn,
	start_behavior::SkillBehavior,
	SkillBehaviorConfig,
};
use serde::{Deserialize, Serialize};
use shape::SkillShapeDto;
use start_behavior::SkillBehaviorDto;

#[derive(Serialize, Deserialize, Debug)]
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

		Self::from_shape(BuildSkillShape::from(value.shape))
			.spawning_on(value.spawn_on)
			.with_contact_behaviors(contact.collect())
			.with_projection_behaviors(projection.collect())
	}
}
