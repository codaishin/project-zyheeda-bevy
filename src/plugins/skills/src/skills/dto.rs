pub(crate) mod run_skill_behavior;

use super::{AnimationStrategy, Skill};
use crate::SkillItemType;
use common::{
	dto::duration::DurationDto,
	traits::{
		load_asset::{LoadAsset, Path},
		register_custom_assets::{AssetFileExtensions, LoadFrom},
	},
};
use run_skill_behavior::RunSkillBehaviorDto;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, time::Duration};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SkillDto {
	name: String,
	cast_time: DurationDto,
	animation: AnimationStrategy,
	behavior: RunSkillBehaviorDto,
	is_usable_with: HashSet<SkillItemType>,
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
			name: skill_data.name,
			cast_time: Duration::from(skill_data.cast_time),
			animation: skill_data.animation,
			behavior: skill_data.behavior.into(),
			is_usable_with: skill_data.is_usable_with,
			icon: skill_data.icon.map(|icon| asset_server.load_asset(icon)),
		}
	}
}
