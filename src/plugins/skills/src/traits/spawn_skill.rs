pub(crate) mod extension;

use crate::skills::shape::OnSkillStop;
use common::{tools::action_key::slot::SlotKey, traits::handles_skill_physics::SkillCaster};

pub(crate) trait SpawnSkill<TSkillConfig> {
	fn spawn_skill(
		&mut self,
		config: TSkillConfig,
		caster: SkillCaster,
		slot: SlotKey,
	) -> OnSkillStop;
}
