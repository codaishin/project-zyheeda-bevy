use crate::{components::skill::Skill, observers::skill_prefab::GetLifetime};
use common::traits::handles_skill_physics::{SkillShape, ground_target::SphereAoE};
use std::time::Duration;

impl GetLifetime for Skill {
	fn get_lifetime(&self) -> Option<Duration> {
		match self.shape {
			SkillShape::SphereAoE(SphereAoE { lifetime, .. }) => lifetime,
			_ => None,
		}
	}
}
