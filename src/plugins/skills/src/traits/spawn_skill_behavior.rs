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
	fn spawn<TLifetimes, TEffects, TShaders>(
		&self,
		commands: &mut TCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> OnSkillStop
	where
		TLifetimes: HandlesLifetime + 'static,
		TEffects: HandlesAllEffects + 'static,
		TShaders: HandlesEffectShadingForAll + 'static;
}
