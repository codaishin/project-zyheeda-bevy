pub(crate) mod attach_effect;
pub(crate) mod spawn;

use crate::behaviors::{
	SkillBehaviorConfig,
	attach_skill_effect::AttachEffect,
	spawn_skill::{SpawnOn, SpawnSkill},
};
use attach_effect::AttachEffectDto;
use serde::{Deserialize, Serialize};
use spawn::SpawnSkillDto;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct SkillBehaviorConfigDto {
	shape: SpawnSkillDto,
	contact: Vec<AttachEffectDto>,
	projection: Vec<AttachEffectDto>,
	spawn_on: SpawnOn,
}

impl From<SkillBehaviorConfigDto> for SkillBehaviorConfig {
	fn from(value: SkillBehaviorConfigDto) -> Self {
		let contact = value.contact.into_iter().map(AttachEffect::from);
		let projection = value.projection.into_iter().map(AttachEffect::from);

		Self {
			shape: SpawnSkill::from(value.shape),
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
			shape: SpawnSkillDto::from(value.shape),
			contact: contact.collect(),
			projection: projection.collect(),
			spawn_on: value.spawn_on,
		}
	}
}
