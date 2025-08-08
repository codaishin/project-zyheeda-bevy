pub mod spawn_ground_target;
pub mod spawn_projectile;
pub mod spawn_shield;

use super::{SkillCaster, SkillTarget};
use crate::traits::skill_builder::{SkillBuilder, SkillShape};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::handles_skill_behaviors::{HandlesSkillBehaviors, SkillSpawner},
};
use spawn_ground_target::SpawnGroundTargetedAoe;
use spawn_projectile::SpawnProjectile;
use spawn_shield::SpawnShield;

#[cfg(test)]
pub(crate) type SpawnSkillFn =
	for<'a> fn(&'a mut Commands, &SkillCaster, SkillSpawner, &SkillTarget) -> SkillShape;

#[derive(Debug, PartialEq, Clone)]
pub enum OnSkillStop {
	Ignore,
	Stop(PersistentEntity),
}

#[derive(Debug, Clone)]
#[cfg_attr(not(test), derive(PartialEq))]
pub(crate) enum BuildSkillShape {
	GroundTargetedAoe(SpawnGroundTargetedAoe),
	Projectile(SpawnProjectile),
	Shield(SpawnShield),
	#[cfg(test)]
	Fn(SpawnSkillFn),
}

#[cfg(test)]
impl Default for BuildSkillShape {
	fn default() -> Self {
		Self::Fn(|commands, _, _, _| {
			let contact = commands.spawn_empty().id();
			let projection = commands.spawn(ChildOf(contact)).id();
			SkillShape {
				contact,
				projection,
				on_skill_stop: OnSkillStop::Ignore,
			}
		})
	}
}

#[cfg(test)]
impl PartialEq for BuildSkillShape {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::GroundTargetedAoe(l0), Self::GroundTargetedAoe(r0)) => l0 == r0,
			(Self::Projectile(l0), Self::Projectile(r0)) => l0 == r0,
			(Self::Shield(l0), Self::Shield(r0)) => l0 == r0,
			(Self::Fn(l0), Self::Fn(r0)) => std::ptr::fn_addr_eq(*l0, *r0),
			_ => false,
		}
	}
}

impl BuildSkillShape {
	#[cfg(test)]
	pub(crate) const NO_SHAPE: BuildSkillShape = BuildSkillShape::Fn(Self::no_shape);

	#[cfg(test)]
	fn no_shape(
		commands: &mut Commands,
		_: &SkillCaster,
		_: SkillSpawner,
		_: &SkillTarget,
	) -> SkillShape {
		let contact = commands.spawn_empty().id();
		let persistent_contact = PersistentEntity::default();
		let projection = commands.spawn((ChildOf(contact), persistent_contact)).id();
		let on_skill_stop = OnSkillStop::Stop(persistent_contact);

		SkillShape {
			contact,
			projection,
			on_skill_stop,
		}
	}

	pub(crate) fn build<TSkillBehaviors>(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: SkillSpawner,
		target: &SkillTarget,
	) -> SkillShape
	where
		TSkillBehaviors: HandlesSkillBehaviors + 'static,
	{
		match self {
			Self::GroundTargetedAoe(gt) => {
				gt.build::<TSkillBehaviors>(commands, caster, spawner, target)
			}
			Self::Projectile(pr) => pr.build::<TSkillBehaviors>(commands, caster, spawner, target),
			Self::Shield(sh) => sh.build::<TSkillBehaviors>(commands, caster, spawner, target),
			#[cfg(test)]
			Self::Fn(func) => func(commands, caster, spawner, target),
		}
	}
}
