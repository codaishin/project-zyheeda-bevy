pub(crate) mod animate_data;
pub(crate) mod run_skill_behavior_data;
pub(crate) mod skill_animation_data;

use super::Skill;
use crate::items::ItemType;
use animate_data::AnimateData;
use common::{
	tools::duration_data::DurationData,
	traits::{
		asset_file_extensions::AssetFileExtensions,
		load_asset::{LoadAsset, Path},
		load_from::LoadFrom,
	},
};
use run_skill_behavior_data::RunSkillBehaviorData;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, time::Duration};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SkillData {
	id: Uuid,
	name: String,
	cast_time: DurationData,
	animate: AnimateData,
	behavior: RunSkillBehaviorData,
	is_usable_with: HashSet<ItemType>,
	icon: Option<Path>,
}

impl AssetFileExtensions for SkillData {
	fn asset_file_extensions() -> &'static [&'static str] {
		&["skill"]
	}
}

impl LoadFrom<SkillData> for Skill {
	fn load_from<TLoadAsset: LoadAsset>(
		skill_data: SkillData,
		asset_server: &mut TLoadAsset,
	) -> Self {
		Self {
			id: skill_data.id,
			name: skill_data.name,
			cast_time: Duration::from(skill_data.cast_time),
			animate: skill_data.animate.into(),
			behavior: skill_data.behavior.into(),
			is_usable_with: skill_data.is_usable_with,
			icon: skill_data.icon.map(|icon| asset_server.load_asset(icon)),
		}
	}
}
