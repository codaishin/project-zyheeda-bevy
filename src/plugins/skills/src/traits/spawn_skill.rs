use crate::behaviors::skill_shape::OnSkillStop;
use common::traits::handles_skill_physics::{
	Contact,
	Projection,
	SkillCaster,
	SkillSpawner,
	SkillTarget,
};

pub(crate) trait SpawnSkill<TSkillConfig> {
	fn spawn_skill(
		&mut self,
		shape: TSkillConfig,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> OnSkillStop;
}

pub(crate) trait SkillContact {
	fn skill_contact(
		&self,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> Contact;
}
pub(crate) trait SkillProjection {
	fn skill_projection(&self) -> Projection;
}
