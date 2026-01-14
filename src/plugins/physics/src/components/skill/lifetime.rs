use crate::{components::skill::Skill, observers::skill_prefab::GetLifetime};
use std::time::Duration;

impl GetLifetime for Skill {
	fn get_lifetime(&self) -> Option<Duration> {
		self.lifetime
	}
}
