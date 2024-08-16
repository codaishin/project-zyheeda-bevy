pub mod spawn_ground_target;

use super::{SkillCaster, SkillSpawner, Target};
use bevy::{
	ecs::system::EntityCommands,
	prelude::{Commands, Entity},
};
use spawn_ground_target::SpawnGroundTarget;

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
	GroundTarget(SpawnGroundTarget),
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
			Self::GroundTarget(gt) => gt.apply(commands, caster, spawn, target),
		}
	}
}
