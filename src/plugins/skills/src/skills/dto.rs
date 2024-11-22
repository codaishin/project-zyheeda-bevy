pub(crate) mod animate;
pub(crate) mod run_skill_behavior;
pub(crate) mod skill_animation;

use super::Skill;
use crate::SkillItemType;
use animate::AnimateDto;
use common::{
	dto::duration::DurationDto,
	traits::load_asset::{LoadAsset, Path},
};
use loading::traits::{asset_file_extensions::AssetFileExtensions, load_from::LoadFrom};
use run_skill_behavior::RunSkillBehaviorDto;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, time::Duration};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SkillDto {
	name: String,
	cast_time: DurationDto,
	animate: AnimateDto,
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
			animate: skill_data.animate.into(),
			behavior: skill_data.behavior.into(),
			is_usable_with: skill_data.is_usable_with,
			icon: skill_data.icon.map(|icon| asset_server.load_asset(icon)),
		}
	}
}
