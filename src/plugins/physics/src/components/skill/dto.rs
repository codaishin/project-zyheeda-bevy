use crate::components::skill::{CreatedFrom, Skill};
use common::{
	errors::Unreachable,
	traits::{
		handles_custom_assets::TryLoadFrom,
		handles_skill_physics::{Contact, Effect, Projection},
	},
};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillDto {
	lifetime: Option<Duration>,
	contact: Contact,
	contact_effects: Vec<Effect>,
	projection: Projection,
	projection_effects: Vec<Effect>,
}

impl From<Skill> for SkillDto {
	fn from(skill: Skill) -> Self {
		Self {
			lifetime: skill.lifetime,
			contact: skill.contact,
			contact_effects: skill.contact_effects,
			projection: skill.projection,
			projection_effects: skill.projection_effects,
		}
	}
}

impl TryLoadFrom<SkillDto> for Skill {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		dto: SkillDto,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError> {
		Ok(Self {
			created_from: CreatedFrom::Save,
			lifetime: dto.lifetime,
			contact: dto.contact,
			contact_effects: dto.contact_effects,
			projection: dto.projection,
			projection_effects: dto.projection_effects,
		})
	}
}
