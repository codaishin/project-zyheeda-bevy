use super::{GetStaticSkillBehavior, RunSkill, SkillBundleConfig};
use crate::skills::{SkillBehavior, SkillCaster, SkillSpawner, Target};
use behaviors::components::GravityWell;

impl SkillBundleConfig for GravityWell {
	type Bundle = GravityWell;

	const STOPPABLE: bool = false;

	fn new_skill_bundle(_: &SkillCaster, _: &SkillSpawner, _: &Target) -> Self::Bundle {
		GravityWell
	}
}

impl GetStaticSkillBehavior for GravityWell {
	fn behavior() -> SkillBehavior {
		SkillBehavior::OnActive(GravityWell::run_skill)
	}
}
