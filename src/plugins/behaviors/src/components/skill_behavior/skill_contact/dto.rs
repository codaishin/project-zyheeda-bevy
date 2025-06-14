use crate::components::skill_behavior::skill_contact::{CreatedFrom, SkillContact};
use common::{
	errors::Unreachable,
	traits::{
		handles_custom_assets::TryLoadFrom,
		handles_skill_behaviors::{Integrity, Motion, Shape},
		load_asset::LoadAsset,
	},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillContactDto {
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

impl TryLoadFrom<SkillContactDto> for SkillContact {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		contact: SkillContactDto,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		Ok(Self {
			shape: contact.shape,
			integrity: contact.integrity,
			motion: contact.motion,
			created_from: CreatedFrom::Save,
		})
	}
}
