use crate::{
	behaviors::{SkillCaster, build_skill_shape::OnSkillStop, spawn_on::SpawnOn},
	components::SkillTarget,
};
use common::traits::{
	handles_effect::HandlesAllEffects,
	handles_skill_behaviors::{HandlesSkillBehaviors, SkillSpawner},
};

pub(crate) trait SpawnSkillBehavior<TCommands> {
	fn spawn_on(&self) -> SpawnOn;
	fn spawn<TEffects, TSkillBehaviors>(
		&self,
		commands: &mut TCommands,
		caster: &SkillCaster,
		spawner: SkillSpawner,
		target: &SkillTarget,
	) -> OnSkillStop
	where
		TEffects: HandlesAllEffects + 'static,
		TSkillBehaviors: HandlesSkillBehaviors + 'static;
}
