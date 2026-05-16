use crate::{SkillDto, components::combo_node::ComboNode, skills::Skill};
use common::{
	errors::Unreachable,
	tools::action_key::slot::SlotKey,
	traits::{handles_custom_assets::TryLoadFrom, load_asset::LoadAsset},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct ComboNodeDto(Vec<(SlotKey, (SkillDto, ComboNodeDto))>);

impl From<ComboNode> for ComboNodeDto {
	fn from(node: ComboNode) -> Self {
		Self(
			node.into_iter()
				.map(|(slot_key, (skill, node))| {
					(slot_key, (SkillDto::from(skill), Self::from(node)))
				})
				.collect(),
		)
	}
}

impl TryLoadFrom<ComboNodeDto> for ComboNode {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		node: ComboNodeDto,
		asset_server: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError>
	where
		TLoadAsset: LoadAsset,
	{
		Ok(Self(
			node.0
				.into_iter()
				.map(|(slot_key, (skill, node))| {
					let Ok(skill) = Skill::try_load_from(skill, asset_server);
					let Ok(node) = Self::try_load_from(node, asset_server);
					(slot_key, (skill, node))
				})
				.collect(),
		))
	}
}
