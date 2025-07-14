pub mod spawn_ground_target;
pub mod spawn_projectile;
pub mod spawn_shield;

use super::{SkillCaster, SkillTarget};
use crate::traits::skill_builder::{SkillBuilder, SkillShape};
use bevy::prelude::*;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::handles_skill_behaviors::{HandlesSkillBehaviors, Spawner},
};
use spawn_ground_target::SpawnGroundTargetedAoe;
use spawn_projectile::SpawnProjectile;
use spawn_shield::SpawnShield;

#[cfg(test)]
pub(crate) type BuildSkillShapeFn =
	for<'a> fn(&'a mut Commands, &SkillCaster, Spawner, &SkillTarget) -> SkillShape;

#[derive(Debug, PartialEq, Clone)]
pub enum OnSkillStop {
	Ignore,
	Stop(PersistentEntity),
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum BuildSkillShape {
	GroundTargetedAoe(SpawnGroundTargetedAoe),
	Projectile(SpawnProjectile),
	Shield(SpawnShield),
	#[cfg(test)]
	Fn(BuildSkillShapeFn),
}

#[cfg(test)]
impl Default for BuildSkillShape {
	fn default() -> Self {
		Self::Fn(|commands, _, _, _| {
			let contact = commands.spawn_empty().id();
			let projection = commands.spawn(ChildOf(contact)).id();
			SkillShape {
				contact,
				projection,
				on_skill_stop: OnSkillStop::Ignore,
			}
		})
	}
}

impl BuildSkillShape {
	#[cfg(test)]
	pub(crate) const NO_SHAPE: BuildSkillShape = BuildSkillShape::Fn(Self::no_shape);

	#[cfg(test)]
	fn no_shape(
		commands: &mut Commands,
		_: &SkillCaster,
		_: Spawner,
		_: &SkillTarget,
	) -> SkillShape {
		let contact = commands.spawn_empty().id();
		let persistent_contact = PersistentEntity::default();
		let projection = commands.spawn((ChildOf(contact), persistent_contact)).id();
		let on_skill_stop = OnSkillStop::Stop(persistent_contact);

		SkillShape {
			contact,
			projection,
			on_skill_stop,
		}
	}

	pub(crate) fn build<TSkillBehaviors>(
		&self,
		commands: &mut Commands,
		caster: &SkillCaster,
		spawner: Spawner,
		target: &SkillTarget,
	) -> SkillShape
	where
		TSkillBehaviors: HandlesSkillBehaviors + 'static,
	{
		match self {
			Self::GroundTargetedAoe(gt) => {
				gt.build::<TSkillBehaviors>(commands, caster, spawner, target)
			}
			Self::Projectile(pr) => pr.build::<TSkillBehaviors>(commands, caster, spawner, target),
			Self::Shield(sh) => sh.build::<TSkillBehaviors>(commands, caster, spawner, target),
			#[cfg(test)]
			Self::Fn(func) => func(commands, caster, spawner, target),
		}
	}
}
