use super::{SkillCaster, SkillSpawner, Target};
use bevy::ecs::system::EntityCommands;

pub type StartBehaviorFn = fn(&mut EntityCommands, &SkillCaster, &SkillSpawner, &Target);

#[derive(Debug, PartialEq, Clone)]
pub enum StartBehavior {
	Fn(StartBehaviorFn),
}

impl StartBehavior {
	pub fn apply(
		&self,
		entity: &mut EntityCommands,
		caster: &SkillCaster,
		spawn: &SkillSpawner,
		target: &Target,
	) {
		match self {
			StartBehavior::Fn(func) => func(entity, caster, spawn, target),
		}
	}
}
