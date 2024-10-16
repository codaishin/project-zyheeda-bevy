pub mod build_skill_shape;
pub mod spawn_on;
pub mod start_behavior;

use crate::{
	skills::{lifetime::LifeTimeDefinition, SelectInfo},
	traits::skill_builder::SkillShape,
};
use bevy::{
	ecs::system::EntityCommands,
	math::Ray3d,
	prelude::{default, Commands, Entity, GlobalTransform},
};
use build_skill_shape::BuildSkillShape;
use common::{components::Outdated, resources::ColliderInfo};
use spawn_on::SpawnOn;
use start_behavior::SkillBehavior;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SkillSpawner(pub Entity);

impl From<Entity> for SkillSpawner {
	fn from(entity: Entity) -> Self {
		Self(entity)
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SkillCaster(pub Entity);

impl From<Entity> for SkillCaster {
	fn from(entity: Entity) -> Self {
		Self(entity)
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
pub struct SkillBehaviorConfig<T> {
	shape: BuildSkillShape<T>,
	contact: Vec<SkillBehavior>,
	projection: Vec<SkillBehavior>,
	pub(crate) spawn_on: SpawnOn,
}

impl<T> SkillBehaviorConfig<T>
where
	LifeTimeDefinition: From<T>,
	T: Clone,
{
	pub(crate) fn from_shape(shape: BuildSkillShape<T>) -> Self {
		Self {
			shape,
			contact: default(),
			projection: default(),
			spawn_on: default(),
		}
	}

	pub(crate) fn spawning_on(self, spawn_on: SpawnOn) -> Self {
		Self {
			shape: self.shape,
			contact: self.contact,
			spawn_on,
			projection: self.projection,
		}
	}

	pub(crate) fn with_contact_behaviors(self, contact: Vec<SkillBehavior>) -> Self {
		Self {
			shape: self.shape,
			contact,
			spawn_on: self.spawn_on,
			projection: self.projection,
		}
	}

	pub(crate) fn with_projection_behaviors(self, projection: Vec<SkillBehavior>) -> Self {
		Self {
			shape: self.shape,
			contact: self.contact,
			spawn_on: self.spawn_on,
			projection,
		}
	}

	pub(crate) fn spawn_shape(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &Target,
	) -> SkillShape {
		self.shape.build(commands, caster, spawner, target)
	}

	pub(crate) fn start_contact_behavior(
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

	pub(crate) fn start_projection_behavior(
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
