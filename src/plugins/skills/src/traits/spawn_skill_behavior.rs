use crate::behaviors::{
	build_skill_shape::OnSkillStop,
	spawn_on::SpawnOn,
	SkillCaster,
	SkillSpawner,
	Target,
};
use common::{
	effects::deal_damage::DealDamage,
	traits::{handles_effect::HandlesEffect, handles_lifetime::HandlesLifetime},
};

pub(crate) trait SpawnSkillBehavior<TCommands> {
	fn spawn_on(&self) -> SpawnOn;
	fn spawn<TLifetimeDependency, TEffectDependency>(
		&self,
		commands: &mut TCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> OnSkillStop
	where
		TLifetimeDependency: HandlesLifetime + 'static,
		TEffectDependency: HandlesEffect<DealDamage> + 'static;
}
