use crate::components::skill::{CreatedFrom, Skill};
use common::{
	errors::Unreachable,
	traits::{
		handles_custom_assets::TryLoadFrom,
		handles_skill_physics::{Effect, SkillCaster, SkillShape, SkillSpawner, SkillTarget},
	},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct SkillDto {
	pub(crate) shape: SkillShape,
	pub(crate) contact_effects: Vec<Effect>,
	pub(crate) projection_effects: Vec<Effect>,
	pub(crate) caster: SkillCaster,
	pub(crate) spawner: SkillSpawner,
	pub(crate) target: SkillTarget,
}

impl From<Skill> for SkillDto {
	fn from(skill: Skill) -> Self {
		Self {
			shape: skill.shape,
			contact_effects: skill.contact_effects,
			projection_effects: skill.projection_effects,
			caster: skill.caster,
			spawner: skill.spawner,
			target: skill.target,
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
			spawner: dto.spawner,
			target: dto.target,
		})
	}
}
