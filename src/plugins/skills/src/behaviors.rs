pub mod attach_skill_effect;
pub mod spawn_skill;

use crate::{
	behaviors::{attach_skill_effect::AttachEffect, spawn_skill::SpawnOn},
	traits::skill_builder::SkillShape,
};
use bevy::prelude::*;
use common::{
	traits::{
		handles_physics::HandlesAllPhysicalEffects,
		handles_skill_physics::{HandlesNewPhysicalSkill, SkillCaster, SkillSpawner, SkillTarget},
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};
use spawn_skill::SpawnSkill;

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
		commands: &mut ZyheedaCommands,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> SkillShape
	where
		TSkillBehaviors: HandlesNewPhysicalSkill + 'static,
	{
		self.shape
			.build::<TSkillBehaviors>(commands, caster, spawner, target)
	}

	pub(crate) fn start_contact_behavior<TEffects>(
		&self,
		entity: &mut ZyheedaEntityCommands,
		caster: SkillCaster,
		target: SkillTarget,
	) where
		TEffects: HandlesAllPhysicalEffects,
	{
		for start in &self.contact {
			start.attach::<TEffects>(entity, caster, target);
		}
	}

	pub(crate) fn start_projection_behavior<TEffects>(
		&self,
		entity: &mut ZyheedaEntityCommands,
		caster: SkillCaster,
		target: SkillTarget,
	) where
		TEffects: HandlesAllPhysicalEffects,
	{
		for start in &self.projection {
			start.attach::<TEffects>(entity, caster, target);
		}
	}
}
