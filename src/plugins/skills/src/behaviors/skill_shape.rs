pub mod beam;
pub mod ground_target;
pub mod projectile;
pub mod shield;

use super::{SkillCaster, SkillTarget};
use crate::{
	behaviors::skill_shape::beam::Beam,
	traits::skill_builder::{SkillBuilder, SkillLayout},
};
use common::{
	components::persistent_entity::PersistentEntity,
	traits::{
		handles_physics::colliders::Blocker,
		handles_skill_physics::{HandlesNewPhysicalSkill, SkillSpawner},
	},
	zyheeda_commands::ZyheedaCommands,
};
use ground_target::GroundTargetedAoe;
use projectile::Projectile;
use serde::{Deserialize, Serialize};
use shield::Shield;
use std::collections::HashSet;

#[cfg(test)]
pub(crate) type SpawnSkillFn =
	fn(&mut ZyheedaCommands, SkillCaster, SkillSpawner, SkillTarget) -> SkillLayout;

#[derive(Debug, Clone)]
#[cfg_attr(not(test), derive(PartialEq))]
pub(crate) enum SkillShape {
	GroundTargetedAoe(GroundTargetedAoe),
	Projectile(Projectile),
	Beam(Beam),
	Shield(Shield),
	#[cfg(test)]
	Fn(SpawnSkillFn), // TODO: REMOVE
}

#[cfg(test)]
impl Default for SkillShape {
	fn default() -> Self {
		use bevy::prelude::*;

		Self::Fn(|commands, _, _, _| {
			let contact = commands.spawn(()).id();
			let projection = commands.spawn(ChildOf(contact)).id();
			SkillLayout {
				contact,
				projection,
				on_skill_stop: OnSkillStop::Ignore,
			}
		})
	}
}

#[cfg(test)]
impl PartialEq for SkillShape {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::GroundTargetedAoe(l0), Self::GroundTargetedAoe(r0)) => l0 == r0,
			(Self::Projectile(l0), Self::Projectile(r0)) => l0 == r0,
			(Self::Shield(l0), Self::Shield(r0)) => l0 == r0,
			(Self::Fn(l0), Self::Fn(r0)) => std::ptr::fn_addr_eq(*l0, *r0),
			_ => false,
		}
	}
}

impl SkillShape {
	#[cfg(test)]
	pub(crate) const NO_SHAPE: SkillShape = SkillShape::Fn(Self::no_shape);

	#[cfg(test)]
	fn no_shape(
		commands: &mut ZyheedaCommands,
		_: SkillCaster,
		_: SkillSpawner,
		_: SkillTarget,
	) -> SkillLayout {
		use bevy::prelude::*;

		let contact = commands.spawn(()).id();
		let persistent_contact = PersistentEntity::default();
		let projection = commands.spawn((ChildOf(contact), persistent_contact)).id();
		let on_skill_stop = OnSkillStop::Stop(persistent_contact);

		SkillLayout {
			contact,
			projection,
			on_skill_stop,
		}
	}

	pub(crate) fn build<TSkillBehaviors>(
		&self,
		commands: &mut ZyheedaCommands,
		caster: SkillCaster,
		spawner: SkillSpawner,
		target: SkillTarget,
	) -> SkillLayout
	where
		TSkillBehaviors: HandlesNewPhysicalSkill + 'static,
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

#[derive(Debug, PartialEq, Clone, Copy)]
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
