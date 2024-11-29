use crate::behaviors::{
	build_skill_shape::OnSkillStop,
	spawn_on::SpawnOn,
	SkillCaster,
	SkillSpawner,
	Target,
};
use common::traits::{
	handles_effect::HandlesAllEffects,
	handles_effect_shading::HandlesEffectShadingForAll,
	handles_lifetime::HandlesLifetime,
};

pub(crate) trait SpawnSkillBehavior<TCommands> {
	fn spawn_on(&self) -> SpawnOn;
	fn spawn<TLifetimeDependency, TEffectDependency, TShaderDependency>(
		&self,
		commands: &mut TCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> OnSkillStop
	where
		TLifetimeDependency: HandlesLifetime + 'static,
		TEffectDependency: HandlesAllEffects + 'static,
		TShaderDependency: HandlesEffectShadingForAll + 'static;
}
