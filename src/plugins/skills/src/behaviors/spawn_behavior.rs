use super::{SkillCaster, SkillSpawner, Target};
use bevy::{
	ecs::system::EntityCommands,
	prelude::{Commands, Entity},
};

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
			SpawnBehavior::Fn(func) => func(commands, caster, spawn, target),
		}
	}
}
