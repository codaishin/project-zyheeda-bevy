use crate::{
	components::{
		asset_model::AssetModel,
		is_blocker::Blocker,
		persistent_entity::PersistentEntity,
	},
	tools::{
		Index,
		Units,
		UnitsPerSecond,
		action_key::slot::{Side, SlotKey},
		iter_helpers::{first, next},
	},
	traits::iteration::{Iter, IterFinite},
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
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, Serialize, Deserialize)]
pub enum SkillSpawner {
	#[default]
	Center,
	Slot(SlotKey),
}

impl IterFinite for SkillSpawner {
	fn iterator() -> Iter<Self> {
		Iter(Some(SkillSpawner::Center))
	}

	fn next(Iter(current): &Iter<Self>) -> Option<Self> {
		match current.as_ref()? {
			SkillSpawner::Center => first(SkillSpawner::Slot),
			SkillSpawner::Slot(key) => next(SkillSpawner::Slot, *key),
		}
	}
}

impl From<SkillSpawner> for Index<usize> {
	fn from(value: SkillSpawner) -> Self {
		match value {
			SkillSpawner::Center => Index(0),
			SkillSpawner::Slot(SlotKey::TopHand(Side::Right)) => Index(1),
			SkillSpawner::Slot(SlotKey::TopHand(Side::Left)) => Index(2),
			SkillSpawner::Slot(SlotKey::BottomHand(Side::Right)) => Index(3),
			SkillSpawner::Slot(SlotKey::BottomHand(Side::Left)) => Index(4),
		}
	}
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub struct ProjectionOffset(pub Vec3);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_spawner() {
		assert_eq!(
			std::iter::empty()
				.chain([SkillSpawner::Center])
				.chain(SlotKey::iterator().map(SkillSpawner::Slot))
				.collect::<Vec<_>>(),
			SkillSpawner::iterator().take(100).collect::<Vec<_>>()
		)
	}

	#[test]
	fn all_indices_different() {
		let count = SkillSpawner::iterator().count();
		let index_count = SkillSpawner::iterator()
			.map(Index::from)
			.collect::<HashSet<_>>()
			.len();

		assert_eq!(count, index_count)
	}
}
