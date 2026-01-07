pub mod attach_skill_effect;
pub mod skill_shape;

use crate::{
	behaviors::{attach_skill_effect::AttachEffect, skill_shape::SpawnOn},
	skills::lifetime_definition::LifeTimeDefinition,
	traits::{
		skill_builder::{SkillLayout, SkillLifetime},
		spawn_skill::{SkillContact, SkillProjection},
	},
};
use bevy::prelude::*;
use common::{
	traits::{
		handles_physics::HandlesAllPhysicalEffects,
		handles_skill_physics::{
			Contact,
			HandlesNewPhysicalSkill,
			Projection,
			SkillCaster,
			SkillSpawner,
			SkillTarget,
		},
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};
use skill_shape::SkillShape;

#[derive(PartialEq, Debug, Clone)]
pub struct SkillBehaviorConfig {
	pub(crate) shape: SkillShape,
	pub(crate) contact: Vec<AttachEffect>,
	pub(crate) projection: Vec<AttachEffect>,
	pub(crate) spawn_on: SpawnOn,
}

impl SkillBehaviorConfig {
	#[cfg(test)]
	pub(crate) fn from_shape(shape: SkillShape) -> Self {
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
	) -> SkillLayout
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

macro_rules! match_shape {
	($config:expr, $callback:expr) => {
		match $config {
			SkillShape::GroundTargetedAoe(shape) => $callback(shape),
			SkillShape::Projectile(shape) => $callback(shape),
			SkillShape::Beam(shape) => $callback(shape),
			SkillShape::Shield(shape) => $callback(shape),
			#[cfg(test)]
			SkillShape::Fn(_) => todo!(), // TODO: REMOVE ARM
		}
	};
}

impl SkillContact for SkillBehaviorConfig {
	fn skill_contact(
		&self,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> Contact {
		match_shape!(&self.shape, |shape| SkillContact::skill_contact(
			shape, caster, spawner, target,
		))
	}
}

impl SkillProjection for SkillBehaviorConfig {
	fn skill_projection(&self) -> Projection {
		match_shape!(&self.shape, SkillProjection::skill_projection)
	}
}

impl SkillLifetime for SkillBehaviorConfig {
	fn lifetime(&self) -> LifeTimeDefinition {
		match_shape!(&self.shape, SkillLifetime::lifetime)
	}
}
