pub(crate) mod extension;

use crate::skills::shape::OnSkillStop;
use common::prelude::*;

pub(crate) trait SpawnSkillInternal<TSkillConfig> {
	fn spawn_skill_internal(
		&mut self,
		config: TSkillConfig,
		caster: SkillCaster,
		slot: SlotKey,
	) -> OnSkillStop;
}
