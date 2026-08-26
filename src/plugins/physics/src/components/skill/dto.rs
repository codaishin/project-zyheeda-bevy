use crate::components::skill::{CreatedFrom, Skill};
use common::prelude::*;
use macros::serde_model;

#[serde_model]
#[derive(Debug, PartialEq)]
pub struct SkillDto {
	pub(crate) shape: SkillShape,
	pub(crate) contact_effects: Vec<SkillEffect>,
	pub(crate) projection_effects: Vec<SkillEffect>,
	pub(crate) caster: SkillCaster,
	pub(crate) mount: SkillMount,
}

impl From<Skill> for SkillDto {
	fn from(skill: Skill) -> Self {
		Self {
			shape: skill.shape,
			contact_effects: skill.contact_effects,
			projection_effects: skill.projection_effects,
			caster: skill.caster,
			mount: skill.mount,
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
			shape: dto.shape,
			created_from: CreatedFrom::Save,
			contact_effects: dto.contact_effects,
			projection_effects: dto.projection_effects,
			caster: dto.caster,
			mount: dto.mount,
		})
	}
}
