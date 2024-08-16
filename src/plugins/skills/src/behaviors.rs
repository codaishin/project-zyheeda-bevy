pub mod spawn_behavior;
pub mod start_behavior;

use crate::skills::SelectInfo;
use bevy::{
	ecs::system::EntityCommands,
	math::Ray3d,
	prelude::{default, Commands, Entity, GlobalTransform},
};
use common::{components::Outdated, resources::ColliderInfo};
use spawn_behavior::{OnSkillStop, SpawnBehavior};
use start_behavior::StartBehavior;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SkillSpawner(pub Entity, pub GlobalTransform);

impl From<Entity> for SkillSpawner {
	fn from(entity: Entity) -> Self {
		Self(entity, default())
	}
}

impl SkillSpawner {
	pub fn with_transform(self, transform: impl Into<GlobalTransform>) -> Self {
		Self(self.0, transform.into())
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SkillCaster(pub Entity, pub GlobalTransform);

impl From<Entity> for SkillCaster {
	fn from(entity: Entity) -> Self {
		Self(entity, default())
	}
}

impl SkillCaster {
	pub fn with_transform(self, transform: impl Into<GlobalTransform>) -> Self {
		Self(self.0, transform.into())
	}
}

pub type Target = SelectInfo<Outdated<GlobalTransform>>;

impl From<Ray3d> for Target {
	fn from(ray: Ray3d) -> Self {
		Self { ray, ..default() }
	}
}

impl Target {
	pub fn with_ray(self, ray: Ray3d) -> Self {
		Self {
			ray,
			collision_info: self.collision_info,
		}
	}

	pub fn with_collision_info(
		self,
		collision_info: Option<ColliderInfo<Outdated<GlobalTransform>>>,
	) -> Self {
		Self {
			ray: self.ray,
			collision_info,
		}
	}
}

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

	pub fn with_start(self, start: Vec<StartBehavior>) -> Self {
		Self {
			spawn: self.spawn,
			start,
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
