pub mod beam;
pub mod ground_target;
pub mod projectile;
pub mod shield;

use crate::skills::shape::beam::Beam;
use common::{
	components::persistent_entity::PersistentEntity,
	traits::handles_physics::physical_bodies::Blocker,
};
use ground_target::GroundTargetedAoe;
use projectile::Projectile;
use serde::{Deserialize, Serialize};
use shield::Shield;
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SkillShape {
	GroundTargetedAoe(GroundTargetedAoe),
	Projectile(Projectile),
	Beam(Beam),
	Shield(Shield),
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
pub(crate) enum Blockers {
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
