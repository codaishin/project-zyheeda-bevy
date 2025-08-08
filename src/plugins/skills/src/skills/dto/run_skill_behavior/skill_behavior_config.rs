pub(crate) mod attach_effect;
pub(crate) mod build_shape;

use crate::behaviors::{
	SkillBehaviorConfig,
	attach_skill_effect::AttachEffect,
	build_skill_shape::BuildSkillShape,
	spawn_on::SpawnOn,
};
use attach_effect::AttachEffectDto;
use build_shape::BuildSkillShapeDto;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct SkillBehaviorConfigDto {
	shape: BuildSkillShapeDto,
	contact: Vec<AttachEffectDto>,
	projection: Vec<AttachEffectDto>,
	spawn_on: SpawnOn,
}

impl From<SkillBehaviorConfigDto> for SkillBehaviorConfig {
	fn from(value: SkillBehaviorConfigDto) -> Self {
		let contact = value.contact.into_iter().map(AttachEffect::from);
		let projection = value.projection.into_iter().map(AttachEffect::from);

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
		let contact = value.contact.into_iter().map(AttachEffectDto::from);
		let projection = value.projection.into_iter().map(AttachEffectDto::from);

		Self {
			shape: BuildSkillShapeDto::from(value.shape),
			contact: contact.collect(),
			projection: projection.collect(),
			spawn_on: value.spawn_on,
		}
	}
}
