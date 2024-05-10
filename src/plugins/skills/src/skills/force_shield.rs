use super::{Cast, Skill};
use crate::traits::SkillTemplate;
use bevy::utils::default;
use std::time::Duration;

pub(crate) struct ForceShield;

impl SkillTemplate for ForceShield {
	fn skill() -> super::Skill {
		Skill {
			name: "force shield",
			cast: Cast {
				pre: Duration::from_millis(100),
				active: Duration::ZERO,
				after: Duration::from_millis(100),
			},
			..default()
		}
	}
}
