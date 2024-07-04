use super::{shoot_hand_gun::ShootHandGun, Animate, Skill};
use crate::{
	items::ItemType,
	traits::{GetSkillAnimation, GetStaticSkillBehavior, SkillTemplate},
};
use behaviors::components::gravity_well::GravityWell;
use common::traits::load_asset::Path;
use std::{collections::HashSet, time::Duration};

pub(crate) struct GravityWellSkill;

impl SkillTemplate for GravityWellSkill {
	fn skill() -> super::Skill {
		Skill {
			name: "gravity well",
			active: Duration::from_millis(200),
			behavior: GravityWell::behavior(),
			// FIXME: introduce cast animation for "magic" like skills
			animate: Animate::Some(ShootHandGun::<()>::animation()),
			is_usable_with: HashSet::from([ItemType::Bracer]),
			icon: Some(|| Path::from("icons/gravity_well.png")),
		}
	}
}
