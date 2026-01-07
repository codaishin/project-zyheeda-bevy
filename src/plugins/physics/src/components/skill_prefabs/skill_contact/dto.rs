use crate::components::skill_prefabs::skill_contact::{CreatedFrom, SkillContact};
use common::{
	errors::Unreachable,
	traits::{
		handles_custom_assets::TryLoadFrom,
		handles_skill_physics::{ContactShape, Motion},
		load_asset::LoadAsset,
	},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SkillContactDto {
	shape: ContactShape,
	motion: Motion,
}

impl From<SkillContact> for SkillContactDto {
	fn from(contact: SkillContact) -> Self {
		Self {
			shape: contact.shape,
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
			motion: contact.motion,
			created_from: CreatedFrom::Save,
		})
	}
}
