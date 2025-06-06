pub mod deal_damage;
pub mod force;
pub mod gravity;

use super::{SkillCaster, SkillTarget};
use bevy::ecs::system::EntityCommands;
use common::traits::{handles_effect::HandlesAllEffects, handles_skill_behaviors::Spawner};
use deal_damage::StartDealingDamage;
use force::StartForce;
use gravity::StartGravity;

#[cfg(test)]
pub type StartBehaviorFn = fn(&mut EntityCommands, &SkillCaster, Spawner, &SkillTarget);

#[derive(Debug, PartialEq, Clone)]
pub enum SkillBehavior {
	Gravity(StartGravity),
	Damage(StartDealingDamage),
	Force(StartForce),
	#[cfg(test)]
	Fn(StartBehaviorFn),
}

impl SkillBehavior {
	pub fn apply<TEffects>(
		&self,
		entity: &mut EntityCommands,
		caster: &SkillCaster,
		spawner: Spawner,
		target: &SkillTarget,
	) where
		TEffects: HandlesAllEffects,
	{
		match self {
			SkillBehavior::Gravity(gr) => gr.apply::<TEffects>(entity, caster, spawner, target),
			SkillBehavior::Damage(dm) => dm.apply::<TEffects>(entity, caster, spawner, target),
			SkillBehavior::Force(fc) => fc.apply::<TEffects>(entity, caster, spawner, target),
			#[cfg(test)]
			SkillBehavior::Fn(func) => func(entity, caster, spawner, target),
		}
	}
}
