use crate::{
	behaviors::{SkillCaster, build_skill_shape::OnSkillStop, spawn_on::SpawnOn},
	components::SkillTarget,
};
use common::{
	traits::{
		handles_effect::HandlesAllEffects,
		handles_skill_behaviors::{HandlesSkillBehaviors, SkillSpawner},
	},
	zyheeda_commands::ZyheedaCommands,
};

pub(crate) trait SpawnSkillBehavior {
	fn spawn_on(&self) -> SpawnOn;
	fn spawn<TEffects, TSkillBehaviors>(
		&self,
		commands: &mut ZyheedaCommands,
		caster: &SkillCaster,
		spawner: SkillSpawner,
		target: &SkillTarget,
	) -> OnSkillStop
	where
		TEffects: HandlesAllEffects + 'static,
		TSkillBehaviors: HandlesSkillBehaviors + 'static;
}
