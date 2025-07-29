pub mod attach_skill_effect;
pub mod spawn_skill;

use crate::{
	behaviors::spawn_skill::SpawnOn,
	components::SkillTarget,
	traits::skill_builder::SkillShape,
};
use attach_skill_effect::AttachEffect;
use bevy::{ecs::system::EntityCommands, prelude::*};
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{
		handles_effect::HandlesAllEffects,
		handles_skill_behaviors::{HandlesSkillBehaviors, Spawner},
	},
};
use spawn_skill::SpawnSkill;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SkillSpawner(pub Entity);

impl From<Entity> for SkillSpawner {
	fn from(entity: Entity) -> Self {
		Self(entity)
	}
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SkillCaster(pub PersistentEntity);

impl From<PersistentEntity> for SkillCaster {
	fn from(persistent: PersistentEntity) -> Self {
		Self(persistent)
	}
}

#[derive(PartialEq, Debug, Clone)]
pub struct SkillBehaviorConfig {
	pub(crate) shape: SpawnSkill,
	pub(crate) contact: Vec<AttachEffect>,
	pub(crate) projection: Vec<AttachEffect>,
	pub(crate) spawn_on: SpawnOn,
}

impl SkillBehaviorConfig {
	#[cfg(test)]
	pub(crate) fn from_shape(shape: SpawnSkill) -> Self {
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
	pub(crate) fn with_contact_behaviors(self, contact: Vec<AttachEffect>) -> Self {
		Self {
			shape: self.shape,
			contact,
			spawn_on: self.spawn_on,
			projection: self.projection,
		}
	}

	#[cfg(test)]
	pub(crate) fn with_projection_behaviors(self, projection: Vec<AttachEffect>) -> Self {
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
		spawner: Spawner,
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
		spawner: Spawner,
		target: &SkillTarget,
	) where
		TEffects: HandlesAllEffects,
	{
		for start in &self.contact {
			start.attach::<TEffects>(entity, caster, spawner, target);
		}
	}

	pub(crate) fn start_projection_behavior<TEffects>(
		&self,
		entity: &mut EntityCommands,
		caster: &SkillCaster,
		spawner: Spawner,
		target: &SkillTarget,
	) where
		TEffects: HandlesAllEffects,
	{
		for start in &self.projection {
			start.attach::<TEffects>(entity, caster, spawner, target);
		}
	}
}
