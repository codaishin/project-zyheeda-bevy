pub mod spawn_ground_target;
pub mod spawn_projectile;
pub mod spawn_shield;

use super::{SkillCaster, SkillSpawner, Target};
use bevy::prelude::{BuildChildren, Commands, Entity};
use spawn_ground_target::SpawnGroundTargetedAoe;
use spawn_projectile::SpawnProjectile;
use spawn_shield::SpawnShield;

pub type SpawnBehaviorFn = for<'a> fn(
	&'a mut Commands,
	&SkillCaster,
	&SkillSpawner,
	&Target,
) -> (Entity, Entity, OnSkillStop);

#[derive(Debug, PartialEq, Clone)]
pub enum OnSkillStop {
	Ignore,
	Stop(Entity),
}

#[derive(PartialEq, Debug, Clone)]
pub enum SkillShape {
	Fn(SpawnBehaviorFn),
	GroundTargetedAoe(SpawnGroundTargetedAoe),
	Projectile(SpawnProjectile),
	Shield(SpawnShield),
}

impl Default for SkillShape {
	fn default() -> Self {
		Self::Fn(|commands, _, _, _| {
			let contact = commands.spawn_empty().id();
			let projection = commands.spawn_empty().set_parent(contact).id();
			(contact, projection, OnSkillStop::Ignore)
		})
	}
}

impl SkillShape {
	pub fn apply(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawn: &SkillSpawner,
		target: &Target,
	) -> (Entity, Entity, OnSkillStop) {
		match self {
			Self::Fn(func) => func(commands, caster, spawn, target),
			Self::GroundTargetedAoe(gt) => gt.apply(commands, caster, spawn, target),
			Self::Projectile(pr) => pr.apply(commands, caster, spawn, target),
			Self::Shield(sh) => sh.apply(commands, caster, spawn, target),
		}
	}
}
