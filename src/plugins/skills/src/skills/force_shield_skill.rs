use super::{shoot_hand_gun::ShootHandGun, Animate, Skill};
use crate::{
	items::ItemType,
	traits::{GetExecution, GetSkillAnimation, SkillTemplate},
};
use behaviors::components::ForceShield;
use std::{collections::HashSet, time::Duration};

pub(crate) struct ForceShieldSkill;

impl SkillTemplate for ForceShieldSkill {
	fn skill() -> super::Skill {
		Skill {
			name: "force shield",
			data: (),
			active: Duration::from_millis(200),
			execution: ForceShield::execution(),
			// FIXME: introduce cast animation for "magic" like skills
			animate: Animate::Some(ShootHandGun::<()>::animation()),
			is_usable_with: HashSet::from([ItemType::Bracer]),
			icon: None,
		}
	}
}
