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

	fn spawn_skill(
		commands: &mut ZyheedaCommands,
		contact: Contact,
		projection: Projection,
	) -> SkillEntities;
}

#[derive(Debug, Clone)]
pub struct Contact {
	pub shape: Shape,
	pub integrity: Integrity,
	pub motion: Motion,
}

#[derive(Debug, Clone)]
pub struct Projection {
	pub shape: Shape,
	pub offset: Option<ProjectionOffset>,
}

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
pub enum Shape {
	Sphere {
		radius: Units,
		hollow_collider: bool,
	},
	Custom {
		model: AssetModel,
		collider: Collider,
		scale: Vec3,
	},
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Integrity {
	Solid,
	Fragile { destroyed_by: HashSet<Blocker> },
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Motion {
	HeldBy {
		caster: PersistentEntity,
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
	Beam {
		caster: PersistentEntity,
		spawner: Spawner,
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
