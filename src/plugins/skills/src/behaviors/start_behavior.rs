pub mod deal_damage;
pub mod force_shield;
pub mod gravity;

use super::{SkillCaster, SkillSpawner, Target};
use bevy::ecs::system::EntityCommands;
use common::traits::{
	handles_effect::HandlesAllEffects,
	handles_effect_shading::HandlesEffectShadingForAll,
};
use deal_damage::StartDealingDamage;
use force_shield::StartForceShield;
use gravity::StartGravity;

#[cfg(test)]
pub type StartBehaviorFn = fn(&mut EntityCommands, &SkillCaster, &SkillSpawner, &Target);

#[derive(Debug, PartialEq, Clone)]
pub enum SkillBehavior {
	Gravity(StartGravity),
	Damage(StartDealingDamage),
	ForceShield(StartForceShield),
	#[cfg(test)]
	Fn(StartBehaviorFn),
}

impl SkillBehavior {
	pub fn apply<TEffectDependency, TShaderDependency>(
		&self,
		entity: &mut EntityCommands,
		caster: &SkillCaster,
		spawn: &SkillSpawner,
		target: &Target,
	) where
		TEffectDependency: HandlesAllEffects,
		TShaderDependency: HandlesEffectShadingForAll,
	{
		match self {
			SkillBehavior::Gravity(gr) => {
				gr.apply::<TEffectDependency, TShaderDependency>(entity, caster, spawn, target)
			}
			SkillBehavior::Damage(dm) => {
				dm.apply::<TEffectDependency>(entity, caster, spawn, target)
			}
			SkillBehavior::ForceShield(fc) => {
				fc.apply::<TShaderDependency>(entity, caster, spawn, target)
			}
			#[cfg(test)]
			SkillBehavior::Fn(func) => func(entity, caster, spawn, target),
		}
	}
}
