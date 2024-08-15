pub mod spawn_behavior;
pub mod start_behavior;

use crate::skills::SelectInfo;
use bevy::{
	ecs::system::EntityCommands,
	prelude::{Commands, Entity, GlobalTransform},
};
use common::components::Outdated;
use spawn_behavior::{OnSkillStop, SpawnBehavior};
use start_behavior::StartBehavior;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SkillSpawner(pub Entity, pub GlobalTransform);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SkillCaster(pub Entity, pub GlobalTransform);

pub type Target = SelectInfo<Outdated<GlobalTransform>>;

#[derive(PartialEq, Debug, Clone)]
pub struct Behavior {
	spawn: SpawnBehavior,
	start: Vec<StartBehavior>,
}

impl Default for Behavior {
	fn default() -> Self {
		Self {
			spawn: SpawnBehavior::Fn(|commands, _, _, _| {
				(commands.spawn_empty(), OnSkillStop::Ignore)
			}),
			start: Default::default(),
		}
	}
}

impl Behavior {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_spawn(self, spawn: SpawnBehavior) -> Self {
		Self {
			spawn,
			start: self.start,
		}
	}

	pub fn with_execute(self, execute: Vec<StartBehavior>) -> Self {
		Self {
			spawn: self.spawn,
			start: execute,
		}
	}

	pub fn spawn<'a>(
		&self,
		commands: &'a mut Commands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> (EntityCommands<'a>, OnSkillStop) {
		self.spawn.apply(commands, caster, spawner, target)
	}

	pub fn start(
		&self,
		entity: &mut EntityCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) {
		for start in &self.start {
			start.apply(entity, caster, spawner, target);
		}
	}
}
