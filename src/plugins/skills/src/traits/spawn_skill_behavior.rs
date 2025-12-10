use crate::behaviors::spawn_skill::{OnSkillStop, SpawnOn};
use common::{
	traits::{
		handles_physics::HandlesAllPhysicalEffects,
		handles_skill_behaviors::{HandlesSkillBehaviors, SkillCaster, SkillSpawner, SkillTarget},
	},
	zyheeda_commands::ZyheedaCommands,
};

pub(crate) trait SpawnSkillBehavior {
	fn spawn_on(&self) -> SpawnOn;
	fn spawn<TPhysics>(
		&self,
		commands: &mut ZyheedaCommands,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> OnSkillStop
	where
		TPhysics: HandlesAllPhysicalEffects + HandlesSkillBehaviors + 'static;
}
