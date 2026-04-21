use crate::{
	assets::agent_config::{
		AgentConfigAsset,
		AgentModel,
		Bones,
		HeightLevels,
		Loadout,
		RequiredClearance,
	},
	components::enemy::void_sphere::VoidSphere,
};
use bevy::prelude::*;
use common::{
	errors::Unreachable,
	traits::{
		handles_animations::{AffectedAnimationBones, Animation, AnimationKey, AnimationMaskBits},
		handles_custom_assets::{AssetFileExtensions, TryLoadFrom},
		handles_movement::MovementSpeed,
		handles_physics::PhysicalDefaultAttributes,
	},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zyheeda_core::serialization::as_vec;

#[derive(TypePath, Debug, PartialEq, Serialize, Deserialize)]
pub struct AgentConfigDto {
	model: ModelConfig,
	loadout: Loadout,
	attributes: PhysicalDefaultAttributes,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum ModelConfig {
	Asset {
		model_path: String,
		bones: Bones,
		movement_speed: MovementSpeed,
		required_clearance: RequiredClearance,
		height_levels: HeightLevels,
		#[serde(with = "as_vec")]
		animations: HashMap<AnimationKey, Animation>,
		animation_mask_groups: HashMap<AnimationMaskBits, AffectedAnimationBones>,
	},
	Procedural(ProceduralModel),
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
		}: AgentConfigDto,
		_: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError> {
		match model {
			ModelConfig::Procedural(ProceduralModel::VoidSphere) => {
				Ok(VoidSphere::config(loadout, attributes))
			}
			ModelConfig::Asset {
				model_path,
				bones,
				movement_speed,
				required_clearance,
				height_levels,
				animations,
				animation_mask_groups,
			} => Ok(AgentConfigAsset {
				loadout,
				bones,
				model: AgentModel::Asset(model_path),
				required_clearance,
				height_levels,
				speed: movement_speed,
				attributes,
				animations,
				animation_mask_groups,
			}),
		}
	}
}

impl AssetFileExtensions for AgentConfigDto {
	fn asset_file_extensions() -> &'static [&'static str] {
		&["agent"]
	}
}
