use crate::{
	behaviors::{
		SkillCaster,
		spawn_skill::{OnSkillStop, SpawnOn},
	},
	components::SkillTarget,
};
use common::{
	traits::{
		handles_physics::HandlesAllPhysicalEffects,
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
		TEffects: HandlesAllPhysicalEffects + 'static,
		TSkillBehaviors: HandlesSkillBehaviors + 'static;
}
