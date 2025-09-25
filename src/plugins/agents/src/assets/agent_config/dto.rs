use crate::{
	assets::agent_config::{AgentConfigAsset, Attributes, Bones, Loadout},
	components::enemy::void_sphere::VoidSphere,
};
use common::{
	errors::Unreachable,
	traits::handles_custom_assets::{AssetFileExtensions, TryLoadFrom},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentConfigAssetDto {
	pub(crate) model: Model,
	pub(crate) loadout: Loadout,
	pub(crate) attributes: Attributes,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum Model {
	Bones(Bones),
	Procedural(ProceduralModel),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum ProceduralModel {
	VoidSphere,
}

impl TryLoadFrom<AgentConfigAssetDto> for AgentConfigAsset {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		from: AgentConfigAssetDto,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError> {
		Ok(AgentConfigAsset {
			loadout: from.loadout,
			bones: match from.model {
				Model::Bones(bones) => bones,
				Model::Procedural(agent_prefab) => match agent_prefab {
					ProceduralModel::VoidSphere => VoidSphere::bones(),
				},
			},
			attributes: from.attributes,
		})
	}
}

impl AssetFileExtensions for AgentConfigAssetDto {
	fn asset_file_extensions() -> &'static [&'static str] {
		&["agent"]
	}
}

pub(crate) trait BonesConfig {
	fn bones() -> Bones;
}
