pub(crate) mod animate_data;
pub(crate) mod projectile_type;
pub(crate) mod skill_animation_data;
pub(crate) mod skill_behavior_data;

use super::Skill;
use crate::items::ItemType;
use animate_data::AnimateData;
use common::traits::load_asset::Path;
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

impl From<SkillData> for Skill {
	fn from(value: SkillData) -> Self {
		Self {
			name: value.name,
			active: value.active,
			animate: value.animate.into(),
			behavior: value.behavior.into(),
			is_usable_with: value.is_usable_with,
			icon: value.icon,
		}
	}
}
