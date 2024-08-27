pub mod spawn_ground_target;
pub mod spawn_projectile;
pub mod spawn_shield;

use super::{SkillCaster, SkillSpawner, Target};
use bevy::{
	ecs::system::EntityCommands,
	prelude::{Commands, Entity},
};
use spawn_ground_target::SpawnGroundTargetedAoe;
use spawn_projectile::SpawnProjectile;
use spawn_shield::SpawnShield;

pub type SpawnBehaviorFn = for<'a> fn(
	&'a mut Commands,
	&SkillCaster,
	&SkillSpawner,
	&Target,
) -> (EntityCommands<'a>, OnSkillStop);

#[derive(Debug, PartialEq, Clone)]
pub enum OnSkillStop {
	Ignore,
	Stop(Entity),
}

#[derive(PartialEq, Debug, Clone)]
pub enum SpawnBehavior {
	Fn(SpawnBehaviorFn),
	GroundTargetedAoe(SpawnGroundTargetedAoe),
	Projectile(SpawnProjectile),
	Shield(SpawnShield),
}

impl SpawnBehavior {
	pub fn apply<'a>(
		&self,
		commands: &'a mut Commands,
		caster: &SkillCaster,
		spawn: &SkillSpawner,
		target: &Target,
	) -> (EntityCommands<'a>, OnSkillStop) {
		match self {
			Self::Fn(func) => func(commands, caster, spawn, target),
			Self::GroundTargetedAoe(gt) => gt.apply(commands, caster, spawn, target),
			Self::Projectile(pr) => pr.apply(commands, caster, spawn, target),
			Self::Shield(sh) => sh.apply(commands, caster, spawn, target),
		}
	}
}
