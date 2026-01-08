pub mod attach_skill_effect;
pub mod skill_shape;

use crate::{
	behaviors::{attach_skill_effect::AttachEffect, skill_shape::SpawnOn},
	skills::lifetime_definition::LifeTimeDefinition,
};
use bevy::prelude::*;
use common::traits::handles_skill_physics::{
	Contact,
	Projection,
	SkillCaster,
	SkillSpawner,
	SkillTarget,
};
use skill_shape::SkillShape;

macro_rules! match_shape {
	($config:expr, $callback:expr) => {
		match $config {
			SkillShape::GroundTargetedAoe(shape) => $callback(shape),
			SkillShape::Projectile(shape) => $callback(shape),
			SkillShape::Beam(shape) => $callback(shape),
			SkillShape::Shield(shape) => $callback(shape),
		}
	};
}

#[derive(PartialEq, Debug, Clone)]
pub struct SkillBehaviorConfig {
	pub(crate) shape: SkillShape,
	pub(crate) contact: Vec<AttachEffect>,
	pub(crate) projection: Vec<AttachEffect>,
	pub(crate) spawn_on: SpawnOn,
}

impl SkillBehaviorConfig {
	#[cfg(test)]
	pub(crate) const fn from_shape(shape: SkillShape) -> Self {
		Self {
			shape,
			contact: vec![],
			projection: vec![],
			spawn_on: SpawnOn::Center,
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

	pub(crate) fn skill_contact(
		&self,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> Contact {
		match_shape!(&self.shape, |shape| SkillContact::skill_contact(
			shape, caster, spawner, target,
		))
	}

	pub(crate) fn skill_projection(&self) -> Projection {
		match_shape!(&self.shape, SkillProjection::skill_projection)
	}

	pub(crate) fn lifetime(&self) -> LifeTimeDefinition {
		match_shape!(&self.shape, SkillLifetime::lifetime)
	}
}

pub(crate) trait SkillContact {
	fn skill_contact(
		&self,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> Contact;
}

pub(crate) trait SkillProjection {
	fn skill_projection(&self) -> Projection;
}

pub(crate) trait SkillLifetime {
	fn lifetime(&self) -> LifeTimeDefinition;
}
