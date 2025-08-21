pub mod force;
pub mod gravity;
pub mod health_damage;

use super::{SkillCaster, SkillTarget};
use common::{traits::handles_effects::HandlesAllEffects, zyheeda_commands::ZyheedaEntityCommands};
use force::AttachForce;
use gravity::AttachGravity;
use health_damage::AttachHealthDamage;

#[cfg(test)]
pub type AttachEffectFn = fn(&mut ZyheedaEntityCommands, &SkillCaster, &SkillTarget);

#[derive(Debug, Clone)]
#[cfg_attr(not(test), derive(PartialEq))]
pub enum AttachEffect {
	Gravity(AttachGravity),
	Damage(AttachHealthDamage),
	Force(AttachForce),
	#[cfg(test)]
	Fn(AttachEffectFn),
}

impl AttachEffect {
	pub fn attach<TEffects>(
		&self,
		entity: &mut ZyheedaEntityCommands,
		caster: &SkillCaster,
		target: &SkillTarget,
	) where
		TEffects: HandlesAllEffects,
	{
		match self {
			AttachEffect::Gravity(gr) => gr.attach::<TEffects>(entity, caster, target),
			AttachEffect::Damage(dm) => dm.attach::<TEffects>(entity, caster, target),
			AttachEffect::Force(fc) => fc.attach::<TEffects>(entity, caster, target),
			#[cfg(test)]
			AttachEffect::Fn(attach) => attach(entity, caster, target),
		}
	}
}

#[cfg(test)]
impl PartialEq for AttachEffect {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Gravity(l0), Self::Gravity(r0)) => l0 == r0,
			(Self::Damage(l0), Self::Damage(r0)) => l0 == r0,
			(Self::Force(l0), Self::Force(r0)) => l0 == r0,
			(Self::Fn(l0), Self::Fn(r0)) => std::ptr::fn_addr_eq(*l0, *r0),
			_ => false,
		}
	}
}
