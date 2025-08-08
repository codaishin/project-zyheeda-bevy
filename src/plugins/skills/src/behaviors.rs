pub mod attach_skill_effect;
pub mod build_skill_shape;
pub mod spawn_on;

use crate::{
	behaviors::{attach_skill_effect::AttachEffect, spawn_on::SpawnOn},
	components::SkillTarget,
	traits::skill_builder::SkillShape,
};
use bevy::{ecs::system::EntityCommands, prelude::*};
use build_skill_shape::BuildSkillShape;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{
		handles_effect::HandlesAllEffects,
		handles_skill_behaviors::{HandlesSkillBehaviors, SkillSpawner},
	},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SkillCaster(pub PersistentEntity);

impl From<PersistentEntity> for SkillCaster {
	fn from(persistent: PersistentEntity) -> Self {
		Self(persistent)
	}
}

#[derive(PartialEq, Debug, Clone)]
pub struct SkillBehaviorConfig {
	pub(crate) shape: BuildSkillShape,
	pub(crate) contact: Vec<AttachEffect>,
	pub(crate) projection: Vec<AttachEffect>,
	pub(crate) spawn_on: SpawnOn,
}

impl SkillBehaviorConfig {
	#[cfg(test)]
	pub(crate) fn from_shape(shape: BuildSkillShape) -> Self {
		Self {
			shape,
			contact: default(),
			projection: default(),
			spawn_on: default(),
		}
	}

	#[cfg(test)]
	pub(crate) fn spawning_on(self, spawn_on: SpawnOn) -> Self {
		Self {
			shape: self.shape,
			contact: self.contact,
			spawn_on,
			projection: self.projection,
		}
	}

	#[cfg(test)]
	pub(crate) fn with_contact_effects(self, contact: Vec<AttachEffect>) -> Self {
		Self {
			shape: self.shape,
			contact,
			spawn_on: self.spawn_on,
			projection: self.projection,
		}
	}

	#[cfg(test)]
	pub(crate) fn with_projection_effects(self, projection: Vec<AttachEffect>) -> Self {
		Self {
			shape: self.shape,
			contact: self.contact,
			spawn_on: self.spawn_on,
			projection,
		}
	}

	pub(crate) fn spawn_shape<TSkillBehaviors>(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: SkillSpawner,
		target: &SkillTarget,
	) -> SkillShape
	where
		TSkillBehaviors: HandlesSkillBehaviors + 'static,
	{
		self.shape
			.build::<TSkillBehaviors>(commands, caster, spawner, target)
	}

	pub(crate) fn start_contact_behavior<TEffects>(
		&self,
		entity: &mut EntityCommands,
		caster: &SkillCaster,
		target: &SkillTarget,
	) where
		TEffects: HandlesAllEffects,
	{
		for start in &self.contact {
			start.attach::<TEffects>(entity, caster, target);
		}
	}

	pub(crate) fn start_projection_behavior<TEffects>(
		&self,
		entity: &mut EntityCommands,
		caster: &SkillCaster,
		target: &SkillTarget,
	) where
		TEffects: HandlesAllEffects,
	{
		for start in &self.projection {
			start.attach::<TEffects>(entity, caster, target);
		}
	}
}
