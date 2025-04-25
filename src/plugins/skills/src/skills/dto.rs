pub(crate) mod run_skill_behavior;

use super::{AnimationStrategy, Skill};
use common::{
	dto::duration::DurationDto,
	tools::item_type::{CompatibleItems, ItemType},
	traits::{
		handles_custom_assets::{AssetFileExtensions, LoadFrom},
		handles_localization::Token,
		load_asset::{LoadAsset, Path},
	},
};
use run_skill_behavior::RunSkillBehaviorDto;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, time::Duration};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SkillDto {
	token: String,
	cast_time: DurationDto,
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

impl LoadFrom<SkillDto> for Skill {
	fn load_from<TLoadAsset: LoadAsset>(
		skill_data: SkillDto,
		asset_server: &mut TLoadAsset,
	) -> Self {
		Self {
			token: Token(skill_data.token),
			cast_time: Duration::from(skill_data.cast_time),
			animation: skill_data.animation,
			behavior: skill_data.behavior.into(),
			compatible_items: CompatibleItems(skill_data.is_usable_with),
			icon: skill_data.icon.map(|icon| asset_server.load_asset(icon)),
		}
	}
}
