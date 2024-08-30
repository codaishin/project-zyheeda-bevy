pub mod spawn_behavior;
pub mod start_behavior;

use crate::skills::SelectInfo;
use bevy::{
	ecs::system::EntityCommands,
	math::Ray3d,
	prelude::{default, Commands, Entity, GlobalTransform},
};
use common::{components::Outdated, resources::ColliderInfo};
use spawn_behavior::{OnSkillStop, SkillShape};
use start_behavior::SkillBehavior;

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

#[derive(Default, PartialEq, Debug, Clone)]
pub struct SkillBehaviorConfig {
	shape: SkillShape,
	contact: Vec<SkillBehavior>,
	projection: Vec<SkillBehavior>,
}

impl SkillBehaviorConfig {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn with_shape(self, shape: SkillShape) -> Self {
		Self {
			shape,
			contact: self.contact,
			projection: self.projection,
		}
	}

	pub fn with_contact_behaviors(self, contact: Vec<SkillBehavior>) -> Self {
		Self {
			shape: self.shape,
			contact,
			projection: self.projection,
		}
	}

	pub fn with_projection_behaviors(self, projection: Vec<SkillBehavior>) -> Self {
		Self {
			shape: self.shape,
			contact: self.contact,
			projection,
		}
	}

	pub fn spawn_shape(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> (Entity, Entity, OnSkillStop) {
		self.shape.apply(commands, caster, spawner, target)
	}

	pub fn start_contact_behavior(
		&self,
		entity: &mut EntityCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) {
		for start in &self.contact {
			start.apply(entity, caster, spawner, target);
		}
	}

	pub fn start_projection_behavior(
		&self,
		entity: &mut EntityCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) {
		for start in &self.projection {
			start.apply(entity, caster, spawner, target);
		}
	}
}
