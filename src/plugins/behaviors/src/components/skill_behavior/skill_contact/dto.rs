use crate::components::skill_behavior::skill_contact::{CreatedFrom, SkillContact};
use common::traits::handles_skill_behaviors::{Integrity, Motion, Shape};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct SkillContactDto {
	shape: Shape,
	integrity: Integrity,
	motion: Motion,
}

impl From<SkillContact> for SkillContactDto {
	fn from(contact: SkillContact) -> Self {
		Self {
			shape: contact.shape,
			integrity: contact.integrity,
			motion: contact.motion,
		}
	}
}

impl From<SkillContactDto> for SkillContact {
	fn from(contact: SkillContactDto) -> Self {
		Self {
			shape: contact.shape,
			integrity: contact.integrity,
			motion: contact.motion,
			created_from: CreatedFrom::Save,
		}
	}
}
