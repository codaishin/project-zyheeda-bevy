use crate::{
	components::{
		asset_model::AssetModel,
		is_blocker::Blocker,
		persistent_entity::PersistentEntity,
	},
	tools::{Index, Units, UnitsPerSecond, action_key::slot::SlotKey},
	zyheeda_commands::ZyheedaCommands,
};
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub trait HandlesSkillBehaviors {
	type TSkillContact: Component;
	type TSkillProjection: Component;
	type TSkillUsage: Component + HoldSkills;

	/// Skills always have a contact and a projection shape.
	///
	/// Activity of those shapes should controlled by their attached effects.
	fn spawn_skill(
		commands: &mut ZyheedaCommands,
		contact: Contact,
		projection: Projection,
	) -> SkillEntities;
}

pub trait HoldSkills {
	type Iter<'a>: Iterator<Item = SlotKey>
	where
		Self: 'a;

	fn holding(&self) -> Self::Iter<'_>;
	fn started_holding(&self) -> Self::Iter<'_>;
}

/// Describes the contact shape of a skill
///
/// These should be used for physical effects like projectile bodies, barriers or beam cores.
#[derive(Debug, Clone)]
pub struct Contact {
	pub shape: ContactShape,
	pub motion: Motion,
}

/// Describes the projection shape of a skill
///
/// These should be used for AoE.
#[derive(Debug, Clone)]
pub struct Projection {
	pub shape: ProjectionShape,
	pub offset: Option<ProjectionOffset>,
}

/// The entities of a spawned skill.
///
/// Skill root should be used to control the lifetime of a skill and can be used to despawn the
/// whole skill.
///
/// Contact components should be added to the contact entity.
///
/// Projection/AoE components should be added to the projection entity.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SkillEntities {
	pub root: SkillRoot,
	pub contact: Entity,
	pub projection: Entity,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SkillRoot {
	pub entity: Entity,
	pub persistent_entity: PersistentEntity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContactShape {
	Sphere {
		radius: Units,
		hollow_collider: bool,
		destroyed_by: HashSet<Blocker>,
	},
	Custom {
		model: AssetModel,
		collider: Collider,
		scale: Vec3,
		destroyed_by: HashSet<Blocker>,
	},
	Beam {
		range: Units,
		radius: Units,
		blocked_by: HashSet<Blocker>,
	},
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProjectionShape {
	Sphere {
		radius: Units,
	},
	Custom {
		model: AssetModel,
		collider: Collider,
		scale: Vec3,
	},
	Beam {
		radius: Units,
	},
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Motion {
	HeldBy {
		caster: PersistentEntity,
		spawner: SkillSpawner,
	},
	Stationary {
		caster: PersistentEntity,
		max_cast_range: Units,
		target_ray: Ray3d,
	},
	Projectile {
		caster: PersistentEntity,
		spawner: SkillSpawner,
		speed: UnitsPerSecond,
		range: Units,
	},
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, Serialize, Deserialize)]
pub enum SkillSpawner {
	#[default]
	Neutral,
	Slot(SlotKey),
}

impl From<SkillSpawner> for Index<usize> {
	fn from(value: SkillSpawner) -> Self {
		match value {
			SkillSpawner::Neutral => Index(0),
			SkillSpawner::Slot(SlotKey(slot)) => Index(slot as usize + 1),
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct ProjectionOffset(pub Vec3);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn to_index() {
		let indices = [
			SkillSpawner::Neutral,
			SkillSpawner::Slot(SlotKey(0)),
			SkillSpawner::Slot(SlotKey(42)),
			SkillSpawner::Slot(SlotKey(255)),
		]
		.into_iter()
		.map(Index::from)
		.collect::<HashSet<_>>();

		assert_eq!(
			HashSet::from([Index(0), Index(1), Index(43), Index(256)]),
			indices
		);
	}
}
