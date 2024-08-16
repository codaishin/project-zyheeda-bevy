pub(crate) mod animate_data;
pub(crate) mod skill_animation_data;
pub(crate) mod skill_behavior_data;

use super::Skill;
use crate::items::ItemType;
use animate_data::AnimateData;
use common::traits::{
	load_asset::{LoadAsset, Path},
	load_from::LoadFrom,
};
use serde::{Deserialize, Serialize};
use skill_behavior_data::SkillBehaviorData;
use std::{collections::HashSet, time::Duration};

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SkillData {
	name: String,
	active: Duration,
	animate: AnimateData,
	behavior: SkillBehaviorData,
	is_usable_with: HashSet<ItemType>,
	icon: Option<Path>,
}

impl LoadFrom<SkillData> for Skill {
	fn load_from<TLoadAsset: LoadAsset>(
		skill_data: SkillData,
		asset_server: &mut TLoadAsset,
	) -> Self {
		Self {
			name: skill_data.name,
			active: skill_data.active,
			animate: skill_data.animate.into(),
			behavior: skill_data.behavior.into(),
			is_usable_with: skill_data.is_usable_with,
			icon: skill_data.icon.map(|icon| asset_server.load_asset(icon)),
		}
	}
}
