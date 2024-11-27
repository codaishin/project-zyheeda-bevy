pub mod spawn_ground_target;
pub mod spawn_projectile;
pub mod spawn_shield;

use super::{SkillCaster, SkillSpawner, Target};
use crate::traits::skill_builder::{SkillBuilder, SkillShape};
use bevy::prelude::{BuildChildren, Commands, Component, Entity};
use spawn_ground_target::SpawnGroundTargetedAoe;
use spawn_projectile::SpawnProjectile;
use spawn_shield::SpawnShield;
use std::time::Duration;

pub(crate) type BuildSkillShapeFn =
	for<'a> fn(&'a mut Commands, &SkillCaster, &SkillSpawner, &Target) -> SkillShape;

#[derive(Debug, PartialEq, Clone)]
pub enum OnSkillStop {
	Ignore,
	Stop(Entity),
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum BuildSkillShape {
	Fn(BuildSkillShapeFn),
	GroundTargetedAoe(SpawnGroundTargetedAoe),
	Projectile(SpawnProjectile),
	Shield(SpawnShield),
}

impl Default for BuildSkillShape {
	fn default() -> Self {
		Self::Fn(|commands, _, _, _| {
			let contact = commands.spawn_empty().id();
			let projection = commands.spawn_empty().set_parent(contact).id();
			SkillShape {
				contact,
				projection,
				on_skill_stop: OnSkillStop::Ignore,
			}
		})
	}
}

impl BuildSkillShape {
	pub(crate) const NO_SHAPE: BuildSkillShape = BuildSkillShape::Fn(Self::no_shape);

	fn no_shape(
		commands: &mut Commands,
		_: &SkillCaster,
		_: &SkillSpawner,
		_: &Target,
	) -> SkillShape {
		let contact = commands.spawn_empty().id();
		let projection = commands.spawn_empty().set_parent(contact).id();
		let on_skill_stop = OnSkillStop::Stop(contact);

		SkillShape {
			contact,
			projection,
			on_skill_stop,
		}
	}

	pub(crate) fn build<TLifeTime>(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawn: &SkillSpawner,
		target: &Target,
	) -> SkillShape
	where
		TLifeTime: From<Duration> + Component,
	{
		match self {
			Self::Fn(func) => func(commands, caster, spawn, target),
			Self::GroundTargetedAoe(gt) => gt.build::<TLifeTime>(commands, caster, spawn, target),
			Self::Projectile(pr) => pr.build::<TLifeTime>(commands, caster, spawn, target),
			Self::Shield(sh) => sh.build::<TLifeTime>(commands, caster, spawn, target),
		}
	}
}
