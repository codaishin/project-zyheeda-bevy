pub mod deal_damage;
pub mod force_shield;
pub mod gravity;

use super::{SkillCaster, SkillSpawner, Target};
use bevy::ecs::system::EntityCommands;
use common::{
	effects::{deal_damage::DealDamage, force_shield::ForceShield},
	traits::{handles_effect::HandlesEffect, handles_effect_shading::HandlesEffectShadingFor},
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
		TEffectDependency: HandlesEffect<DealDamage>,
		TShaderDependency: HandlesEffectShadingFor<ForceShield>,
	{
		match self {
			SkillBehavior::Gravity(gr) => gr.apply(entity, caster, spawn, target),
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
