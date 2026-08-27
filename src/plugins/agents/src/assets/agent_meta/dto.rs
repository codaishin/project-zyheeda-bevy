use crate::{
	assets::agent_meta::{AgentMeta, AgentModel, Bones, HeightLevels, Loadout, RequiredClearance},
	components::enemy::void_sphere::VoidSphere,
};
use bevy::prelude::*;
use common::prelude::*;
use macros::serde_model;
use std::collections::HashMap;
use zyheeda_core::prelude::*;

#[serde_model]
#[derive(TypePath, Debug, PartialEq)]
pub struct AgentConfigDto {
	model: ModelConfig,
	loadout: Loadout,
	attributes: PhysicalDefaultAttributes,
}

#[serde_model]
#[derive(Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub(crate) enum ModelConfig {
	Asset {
		model_path: String,
		interactive_detection_shape: InteractiveFrame,
		bones: Bones,
		movement_speed: MovementSpeed,
		required_clearance: RequiredClearance,
		height_levels: HeightLevels,
		self_skill_scale: Option<Scale<3>>,
		#[serde(with = "hashmap_as_vec")]
		animations: HashMap<AnimationKey, Animation<AnimationNames>>,
		animation_mask_groups: HashMap<AnimationMaskBits, AffectedAnimationBones>,
	},
	Procedural(ProceduralModel),
}

#[serde_model]
#[derive(Debug, PartialEq)]
pub(crate) enum ProceduralModel {
	VoidSphere,
}

impl TryLoadFrom<AgentConfigDto> for AgentMeta {
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
				interactive_detection_shape,
				bones,
				movement_speed,
				required_clearance,
				height_levels,
				self_skill_scale,
				animations,
				animation_mask_groups,
			} => Ok(AgentMeta {
				loadout,
				bones,
				model: AgentModel::Asset(model_path),
				interactive_detection_shape,
				required_clearance,
				height_levels,
				self_skill_scale: self_skill_scale.unwrap_or_default(),
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
