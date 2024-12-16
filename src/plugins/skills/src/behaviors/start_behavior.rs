pub mod deal_damage;
pub mod force_shield;
pub mod gravity;

use super::{SkillCaster, SkillSpawner, SkillTarget};
use bevy::ecs::system::EntityCommands;
use common::traits::handles_effect::HandlesAllEffects;
use deal_damage::StartDealingDamage;
use force_shield::StartForceShield;
use gravity::StartGravity;

#[cfg(test)]
pub type StartBehaviorFn = fn(&mut EntityCommands, &SkillCaster, &SkillSpawner, &SkillTarget);

#[derive(Debug, PartialEq, Clone)]
pub enum SkillBehavior {
	Gravity(StartGravity),
	Damage(StartDealingDamage),
	ForceShield(StartForceShield),
	#[cfg(test)]
	Fn(StartBehaviorFn),
}

impl SkillBehavior {
	pub fn apply<TEffects>(
		&self,
		entity: &mut EntityCommands,
		caster: &SkillCaster,
		spawn: &SkillSpawner,
		target: &SkillTarget,
	) where
		TEffects: HandlesAllEffects,
	{
		match self {
			SkillBehavior::Gravity(gr) => gr.apply::<TEffects>(entity, caster, spawn, target),
			SkillBehavior::Damage(dm) => dm.apply::<TEffects>(entity, caster, spawn, target),
			SkillBehavior::ForceShield(fc) => fc.apply::<TEffects>(entity, caster, spawn, target),
			#[cfg(test)]
			SkillBehavior::Fn(func) => func(entity, caster, spawn, target),
		}
	}
}
