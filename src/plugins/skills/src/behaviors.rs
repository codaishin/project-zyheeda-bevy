pub mod build_skill_shape;
pub mod spawn_on;
pub mod start_behavior;

use crate::{components::SkillTarget, traits::skill_builder::SkillShape};
use bevy::{ecs::system::EntityCommands, prelude::*};
use build_skill_shape::BuildSkillShape;
use common::traits::{
	handles_effect::HandlesAllEffects,
	handles_lifetime::HandlesLifetime,
	handles_skill_behaviors::HandlesSkillBehaviors,
};
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

#[derive(PartialEq, Debug, Clone)]
pub struct SkillBehaviorConfig {
	shape: BuildSkillShape,
	contact: Vec<SkillBehavior>,
	projection: Vec<SkillBehavior>,
	pub(crate) spawn_on: SpawnOn,
}

impl SkillBehaviorConfig {
	pub(crate) fn from_shape(shape: BuildSkillShape) -> Self {
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

	pub(crate) fn spawn_shape<TLifeCycles, TSkillBehaviors>(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &SkillTarget,
	) -> SkillShape
	where
		TLifeCycles: HandlesLifetime,
		TSkillBehaviors: HandlesSkillBehaviors + 'static,
	{
		self.shape
			.build::<TLifeCycles, TSkillBehaviors>(commands, caster, spawner, target)
	}

	pub(crate) fn start_contact_behavior<TEffects>(
		&self,
		entity: &mut EntityCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &SkillTarget,
	) where
		TEffects: HandlesAllEffects,
	{
		for start in &self.contact {
			start.apply::<TEffects>(entity, caster, spawner, target);
		}
	}

	pub(crate) fn start_projection_behavior<TEffects>(
		&self,
		entity: &mut EntityCommands,
		caster: &SkillCaster,
		spawner: &SkillSpawner,
		target: &SkillTarget,
	) where
		TEffects: HandlesAllEffects,
	{
		for start in &self.projection {
			start.apply::<TEffects>(entity, caster, spawner, target);
		}
	}
}
