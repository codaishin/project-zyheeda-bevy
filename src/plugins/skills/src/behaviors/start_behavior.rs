pub mod force;
pub mod start_deal_damage;
pub mod start_gravity;

use super::{SkillCaster, SkillSpawner, Target};
use bevy::ecs::system::EntityCommands;
use force::StartForce;
use start_deal_damage::StartDealingDamage;
use start_gravity::StartGravity;

#[cfg(test)]
pub type StartBehaviorFn = fn(&mut EntityCommands, &SkillCaster, &SkillSpawner, &Target);

#[derive(Debug, PartialEq, Clone)]
pub enum SkillBehavior {
	Gravity(StartGravity),
	Damage(StartDealingDamage),
	Force(StartForce),
	#[cfg(test)]
	Fn(StartBehaviorFn),
}

impl SkillBehavior {
	pub fn apply(
		&self,
		entity: &mut EntityCommands,
		caster: &SkillCaster,
		spawn: &SkillSpawner,
		target: &Target,
	) {
		match self {
			SkillBehavior::Gravity(gr) => gr.apply(entity, caster, spawn, target),
			SkillBehavior::Damage(dm) => dm.apply(entity, caster, spawn, target),
			SkillBehavior::Force(fc) => fc.apply(entity, caster, spawn, target),
			#[cfg(test)]
			SkillBehavior::Fn(func) => func(entity, caster, spawn, target),
		}
	}
}
