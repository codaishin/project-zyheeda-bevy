use crate::{
	assets::agent_config::{AgentConfigAsset, AgentModel, Bones, Loadout},
	components::enemy::void_sphere::VoidSphere,
};
use common::{
	errors::Unreachable,
	traits::{
		animation::{Animation2, AnimationKey},
		handles_custom_assets::{AssetFileExtensions, TryLoadFrom},
		handles_physics::PhysicalDefaultAttributes,
	},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zyheeda_core::serialization::as_vec;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentConfigAssetDto {
	pub(crate) model: Model,
	pub(crate) loadout: Loadout,
	pub(crate) attributes: PhysicalDefaultAttributes,
	#[serde(with = "as_vec")]
	pub(crate) animations: HashMap<AnimationKey, Animation2>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Model {
	Asset { model_path: String, bones: Bones },
	Procedural(ProceduralModel),
}

impl Model {
	fn definition(self) -> (AgentModel, Bones) {
		let procedural = match self {
			Model::Asset { model_path, bones } => return (AgentModel::Asset(model_path), bones),
			Model::Procedural(proc) => proc,
		};

		match procedural {
			ProceduralModel::VoidSphere => (
				AgentModel::Procedural(|e| {
					e.try_insert(VoidSphere);
				}),
				VoidSphere::bones(),
			),
		}
	}
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum ProceduralModel {
	VoidSphere,
}

impl TryLoadFrom<AgentConfigAssetDto> for AgentConfigAsset {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		AgentConfigAssetDto {
			model,
			loadout,
			attributes,
			animations,
		}: AgentConfigAssetDto,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError> {
		let (agent_model, bones) = model.definition();

		Ok(AgentConfigAsset {
			loadout,
			bones,
			agent_model,
			attributes,
			animations,
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
