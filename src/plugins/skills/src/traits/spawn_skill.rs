mod extension;

use crate::behaviors::skill_shape::OnSkillStop;
use common::traits::handles_skill_physics::{SkillCaster, SkillSpawner, SkillTarget};

pub(crate) trait SpawnSkill<TSkillConfig> {
	fn spawn_skill(
		&mut self,
		config: TSkillConfig,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> OnSkillStop;
}
