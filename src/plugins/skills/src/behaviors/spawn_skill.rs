pub mod spawn_beam;
pub mod spawn_ground_target;
pub mod spawn_projectile;
pub mod spawn_shield;

use super::{SkillCaster, SkillTarget};
use crate::{
	behaviors::spawn_skill::spawn_beam::SpawnBeam,
	traits::skill_builder::{SkillBuilder, SkillShape},
};
use bevy::prelude::*;
use common::{
	components::{is_blocker::Blocker, persistent_entity::PersistentEntity},
	traits::handles_skill_behaviors::{HandlesSkillBehaviors, Spawner},
};
use serde::{Deserialize, Serialize};
use spawn_ground_target::SpawnGroundTargetedAoe;
use spawn_projectile::SpawnProjectile;
use spawn_shield::SpawnShield;
use std::collections::HashSet;

#[cfg(test)]
pub(crate) type SpawnSkillFn =
	for<'a> fn(&'a mut Commands, &SkillCaster, Spawner, &SkillTarget) -> SkillShape;

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum SpawnSkill {
	GroundTargetedAoe(SpawnGroundTargetedAoe),
	Projectile(SpawnProjectile),
	Beam(SpawnBeam),
	Shield(SpawnShield),
	#[cfg(test)]
	Fn(SpawnSkillFn),
}

#[cfg(test)]
impl Default for SpawnSkill {
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

impl SpawnSkill {
	#[cfg(test)]
	pub(crate) const NO_SHAPE: SpawnSkill = SpawnSkill::Fn(Self::no_shape);

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
			Self::Beam(bm) => bm.build::<TSkillBehaviors>(commands, caster, spawner, target),
			Self::Shield(sh) => sh.build::<TSkillBehaviors>(commands, caster, spawner, target),
			#[cfg(test)]
			Self::Fn(func) => func(commands, caster, spawner, target),
		}
	}
}

#[derive(Default, PartialEq, Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SpawnOn {
	#[default]
	Center,
	Slot,
}

#[derive(Debug, PartialEq, Clone)]
pub enum OnSkillStop {
	Ignore,
	Stop(PersistentEntity),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
enum Blockers {
	All,
	AnyOf(HashSet<Blocker>),
}

impl From<Blockers> for HashSet<Blocker> {
	fn from(value: Blockers) -> Self {
		match value {
			Blockers::All => Blocker::all(),
			Blockers::AnyOf(blockers) => blockers,
		}
	}
}
