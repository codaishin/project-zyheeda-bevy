use crate::{
	assets::agent_config::{AgentConfigAsset, AgentModel, Bones, Loadout},
	components::enemy::void_sphere::VoidSphere,
};
use bevy::prelude::*;
use common::{
	errors::Unreachable,
	traits::{
		handles_animations::{AffectedAnimationBones, Animation, AnimationKey, AnimationMaskBits},
		handles_custom_assets::{AssetFileExtensions, TryLoadFrom},
		handles_physics::PhysicalDefaultAttributes,
	},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zyheeda_core::serialization::as_vec;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentConfigDto {
	model: Model,
	loadout: Loadout,
	attributes: PhysicalDefaultAttributes,
	#[serde(with = "as_vec")]
	animations: HashMap<AnimationKey, Animation>,
	animation_mask_groups: HashMap<AnimationMaskBits, AffectedAnimationBones>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum Model {
	Asset {
		model_path: String,
		bones: Bones,
		ground_offset: Vec3,
	},
	Procedural(ProceduralModel),
}

impl Model {
	fn definition(self) -> (AgentModel, Bones, Vec3) {
		let procedural = match self {
			Model::Asset {
				model_path,
				bones,
				ground_offset,
			} => return (AgentModel::Asset(model_path), bones, ground_offset),
			Model::Procedural(proc) => proc,
		};

		match procedural {
			ProceduralModel::VoidSphere => (
				AgentModel::Procedural(|e| {
					e.try_insert(VoidSphere);
				}),
				VoidSphere::bones(),
				VoidSphere::GROUND_OFFSET,
			),
		}
	}
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) enum ProceduralModel {
	VoidSphere,
}

impl TryLoadFrom<AgentConfigDto> for AgentConfigAsset {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset>(
		AgentConfigDto {
			model,
			loadout,
			attributes,
			animations,
			animation_mask_groups,
		}: AgentConfigDto,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError> {
		let (agent_model, bones, ground_offset) = model.definition();

		Ok(AgentConfigAsset {
			loadout,
			bones,
			agent_model,
			ground_offset,
			attributes,
			animations,
			animation_mask_groups,
		})
	}
}

impl AssetFileExtensions for AgentConfigDto {
	fn asset_file_extensions() -> &'static [&'static str] {
		&["agent"]
	}
}
