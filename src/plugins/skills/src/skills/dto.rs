pub(crate) mod run_skill_behavior;

use super::{AnimationStrategy, Skill};
use crate::skills::RunSkillBehavior;
use bevy::asset::{AssetPath, Handle};
use common::{
	dto::duration_in_seconds::DurationInSeconds,
	errors::Unreachable,
	tools::item_type::{CompatibleItems, ItemType},
	traits::{
		handles_custom_assets::{AssetFileExtensions, TryLoadFrom},
		handles_localization::Token,
		load_asset::{LoadAsset, Path},
	},
};
use run_skill_behavior::RunSkillBehaviorDto;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, time::Duration};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub(crate) struct SkillDto {
	token: String,
	cast_time: DurationInSeconds,
	animation: AnimationStrategy,
	behavior: RunSkillBehaviorDto,
	is_usable_with: HashSet<ItemType>,
	icon: Option<Path>,
}

impl AssetFileExtensions for SkillDto {
	fn asset_file_extensions() -> &'static [&'static str] {
		&["skill"]
	}
}

impl TryLoadFrom<SkillDto> for Skill {
	type TInstantiationError = Unreachable;

	fn try_load_from<TLoadAsset: LoadAsset>(
		skill_data: SkillDto,
		asset_server: &mut TLoadAsset,
	) -> Result<Self, Self::TInstantiationError> {
		Ok(Self {
			token: Token::from(skill_data.token),
			cast_time: Duration::from(skill_data.cast_time),
			animation: skill_data.animation,
			behavior: RunSkillBehavior::from(skill_data.behavior),
			compatible_items: CompatibleItems(skill_data.is_usable_with),
			icon: match skill_data.icon {
				Some(icon) => asset_server.load_asset(icon),
				None => Handle::default(),
			},
		})
	}
}

impl From<Skill> for SkillDto {
	fn from(skill: Skill) -> Self {
		Self {
			token: (*skill.token).to_owned(),
			cast_time: DurationInSeconds::from(skill.cast_time),
			animation: skill.animation,
			behavior: RunSkillBehaviorDto::from(skill.behavior),
			is_usable_with: skill.compatible_items.0,
			icon: skill.icon.path().map(AssetPath::to_string).map(Path::from),
		}
	}
}
