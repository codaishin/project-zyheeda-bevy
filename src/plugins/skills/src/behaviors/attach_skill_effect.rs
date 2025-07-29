pub mod deal_damage;
pub mod force;
pub mod gravity;

use super::{SkillCaster, SkillTarget};
use bevy::ecs::system::EntityCommands;
use common::traits::{handles_effect::HandlesAllEffects, handles_skill_behaviors::Spawner};
use deal_damage::AttachDealingDamage;
use force::AttachForce;
use gravity::AttachGravity;

#[cfg(test)]
pub type AttachEffectFn = fn(&mut EntityCommands, &SkillCaster, Spawner, &SkillTarget);

#[derive(Debug, PartialEq, Clone)]
pub enum AttachEffect {
	Gravity(AttachGravity),
	Damage(AttachDealingDamage),
	Force(AttachForce),
	#[cfg(test)]
	Fn(AttachEffectFn),
}

impl AttachEffect {
	pub fn attach<TEffects>(
		&self,
		entity: &mut EntityCommands,
		caster: &SkillCaster,
		spawner: Spawner,
		target: &SkillTarget,
	) where
		TEffects: HandlesAllEffects,
	{
		match self {
			AttachEffect::Gravity(gr) => gr.attach::<TEffects>(entity, caster, spawner, target),
			AttachEffect::Damage(dm) => dm.attach::<TEffects>(entity, caster, spawner, target),
			AttachEffect::Force(fc) => fc.attach::<TEffects>(entity, caster, spawner, target),
			#[cfg(test)]
			AttachEffect::Fn(attach) => attach(entity, caster, spawner, target),
		}
	}
}
