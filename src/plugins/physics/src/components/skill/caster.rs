use crate::{components::skill::Skill, observers::skill_prefab::GetCaster};
use common::prelude::*;

impl GetCaster for Skill {
	fn get_caster(&self) -> PersistentEntity {
		let SkillCaster(caster) = self.caster;

		caster
	}
}
