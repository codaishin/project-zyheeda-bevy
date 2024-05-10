use super::{Animate, Cast, Skill, SkillExecution};
use crate::{
	items::ItemType,
	traits::{GetExecution, RunSkill, SkillTemplate},
};
use behaviors::components::ForceShield;
use std::{collections::HashSet, time::Duration};

pub(crate) struct ForceShieldSkill;

impl SkillTemplate for ForceShieldSkill {
	fn skill() -> super::Skill {
		Skill {
			name: "force shield",
			data: (),
			cast: Cast {
				pre: Duration::from_millis(100),
				active: Duration::ZERO,
				after: Duration::from_millis(100),
			},
			execution: ForceShield::execution(),
			animate: Animate::None,
			is_usable_with: HashSet::from([ItemType::Bracer]),
			icon: None,
		}
	}
}

impl GetExecution for ForceShield {
	fn execution() -> SkillExecution {
		SkillExecution {
			run_fn: Some(ForceShield::run_skill),
			execution_stop_on_skill_stop: true,
		}
	}
}
