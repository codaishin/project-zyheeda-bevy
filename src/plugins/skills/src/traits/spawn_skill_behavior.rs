use crate::behaviors::{
	build_skill_shape::OnSkillStop,
	spawn_on::SpawnOn,
	SkillCaster,
	SkillSpawner,
};
use behaviors::components::skill_behavior::SkillTarget;
use common::traits::{handles_effect::HandlesAllEffects, handles_lifetime::HandlesLifetime};

pub(crate) trait SpawnSkillBehavior<TCommands> {
	fn spawn_on(&self) -> SpawnOn;
	fn spawn<TLifetimes, TEffects>(
		&self,
		commands: &mut TCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &SkillTarget,
	) -> OnSkillStop
	where
		TLifetimes: HandlesLifetime + 'static,
		TEffects: HandlesAllEffects + 'static;
}
