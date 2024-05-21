use super::{shoot_hand_gun::ShootHandGun, Animate, Skill};
use crate::{
	items::ItemType,
	traits::{GetSkillAnimation, GetStaticSkillBehavior, SkillTemplate},
};
use behaviors::components::ForceShield;
use common::traits::load_asset::Path;
use std::{collections::HashSet, time::Duration};

pub(crate) struct ForceShieldSkill;

impl SkillTemplate for ForceShieldSkill {
	fn skill() -> super::Skill {
		Skill {
			name: "force shield",
			data: (),
			active: Duration::from_millis(200),
			behavior: ForceShield::behavior(),
			// FIXME: introduce cast animation for "magic" like skills
			animate: Animate::Some(ShootHandGun::<()>::animation()),
			is_usable_with: HashSet::from([ItemType::Bracer]),
			icon: Some(|| Path::from("icons/force_shield.png")),
		}
	}
}
