use crate::behaviors::skill_shape::{OnSkillStop, SpawnOn};
use common::{
	traits::{
		handles_physics::HandlesAllPhysicalEffects,
		handles_skill_physics::{HandlesNewPhysicalSkill, SkillCaster, SkillSpawner, SkillTarget},
	},
	zyheeda_commands::ZyheedaCommands,
};

pub(crate) trait SpawnLoadoutSkill {
	fn spawn_on(&self) -> SpawnOn;
	fn spawn<TPhysics>(
		&self,
		commands: &mut ZyheedaCommands,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> OnSkillStop
	where
		TPhysics: HandlesAllPhysicalEffects + HandlesNewPhysicalSkill + 'static;
}
