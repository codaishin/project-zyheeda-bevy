use crate::behaviors::{
	build_skill_shape::OnSkillStop,
	spawn_on::SpawnOn,
	SkillCaster,
	SkillSpawner,
	Target,
};

pub(crate) trait SpawnSkillBehavior<TCommands> {
	fn spawn_on(&self) -> SpawnOn;
	fn spawn(
		&self,
		commands: &mut TCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> OnSkillStop;
}
