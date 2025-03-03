use crate::{
	behaviors::{SkillCaster, SkillSpawner, build_skill_shape::OnSkillStop, spawn_on::SpawnOn},
	components::SkillTarget,
};
use common::traits::{
	handles_effect::HandlesAllEffects,
	handles_lifetime::HandlesLifetime,
	handles_skill_behaviors::HandlesSkillBehaviors,
};

pub(crate) trait SpawnSkillBehavior<TCommands> {
	fn spawn_on(&self) -> SpawnOn;
	fn spawn<TLifetimes, TEffects, TSkillBehaviors>(
		&self,
		commands: &mut TCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &SkillTarget,
	) -> OnSkillStop
	where
		TLifetimes: HandlesLifetime + 'static,
		TEffects: HandlesAllEffects + 'static,
		TSkillBehaviors: HandlesSkillBehaviors + 'static;
}
