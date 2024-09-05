pub mod spawn_ground_target;
pub mod spawn_projectile;
pub mod spawn_shield;

use super::{SkillCaster, SkillSpawner, Target};
use crate::{
	skills::lifetime::LifeTimeDefinition,
	traits::skill_builder::{SkillBuilder, SkillShape},
};
use bevy::prelude::{BuildChildren, Commands, Entity};
use spawn_ground_target::SpawnGroundTargetedAoe;
use spawn_projectile::SpawnProjectile;
use spawn_shield::SpawnShield;

pub(crate) type BuildSkillShapeFn =
	for<'a> fn(&'a mut Commands, &SkillCaster, &SkillSpawner, &Target) -> SkillShape;

#[derive(Debug, PartialEq, Clone)]
pub enum OnSkillStop {
	Ignore,
	Stop(Entity),
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum BuildSkillShape<TLifeTime> {
	Fn(BuildSkillShapeFn),
	GroundTargetedAoe(SpawnGroundTargetedAoe<TLifeTime>),
	Projectile(SpawnProjectile),
	Shield(SpawnShield),
}

impl<T> Default for BuildSkillShape<T> {
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

impl<TLifeTime> BuildSkillShape<TLifeTime>
where
	LifeTimeDefinition: From<TLifeTime>,
	TLifeTime: Clone,
{
	pub(crate) const NO_SHAPE: BuildSkillShape<TLifeTime> = BuildSkillShape::Fn(Self::no_shape);

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

	pub(crate) fn build(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawn: &SkillSpawner,
		target: &Target,
	) -> SkillShape {
		match self {
			Self::Fn(func) => func(commands, caster, spawn, target),
			Self::GroundTargetedAoe(gt) => gt.build(commands, caster, spawn, target),
			Self::Projectile(pr) => pr.build(commands, caster, spawn, target),
			Self::Shield(sh) => sh.build(commands, caster, spawn, target),
		}
	}
}
