use crate::{
	components::{asset_model::AssetModel, persistent_entity::PersistentEntity},
	tools::{Index, Units, UnitsPerSecond, action_key::slot::SlotKey},
	traits::{
		accessors::get::GetContextMut,
		handles_physics::colliders::{Blocker, Shape},
	},
	zyheeda_commands::ZyheedaCommands,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, ops::Deref};

pub trait HandlesSkillBehaviors {
	type TSkillContact: Component;
	type TSkillProjection: Component;
	type TSkillSpawnerMut<'w, 's>: SystemParam
		+ for<'c> GetContextMut<NewSkill, TContext<'c>: SpawnNewSkill>;

	/// Skills always have a contact and a projection shape.
	///
	/// Activity of those shapes should be controlled by their attached effects.
	fn spawn_skill(
		commands: &mut ZyheedaCommands,
		contact: Contact,
		projection: Projection,
	) -> SkillEntities;
}

pub trait SpawnNewSkill {
	fn spawn_new_skill(&mut self, contact: Contact, projection: Projection) -> SkillEntities;
}

pub struct NewSkill;

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
/// - `root`: The skill's top level entity (can be used to despawn the skill)
/// - `contact`: add/remove contact effect components (projectiles, force fields, ..)
/// - `projection`: add/remove projection/AoE effect components (gravity fields, explosions, ..)
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
		model_scale: Vec3,
		collider: Shape,
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
		model_scale: Vec3,
		collider: Shape,
	},
	Beam {
		radius: Units,
	},
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Motion {
	HeldBy {
		caster: SkillCaster,
		spawner: SkillSpawner,
	},
	Stationary {
		caster: SkillCaster,
		max_cast_range: Units,
		target: SkillTarget,
	},
	Projectile {
		caster: SkillCaster,
		spawner: SkillSpawner,
		speed: UnitsPerSecond,
		range: Units,
	},
}

#[derive(Debug, PartialEq, Clone, Copy, Default, Serialize, Deserialize)]
pub struct SkillCaster(pub PersistentEntity);

impl Deref for SkillCaster {
	type Target = PersistentEntity;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum SkillTarget {
	Ground(Vec3),
	Entity(PersistentEntity),
}

impl Default for SkillTarget {
	fn default() -> Self {
		Self::Ground(Vec3::default())
	}
}

impl From<Vec3> for SkillTarget {
	fn from(ground: Vec3) -> Self {
		Self::Ground(ground)
	}
}

impl From<PersistentEntity> for SkillTarget {
	fn from(entity: PersistentEntity) -> Self {
		Self::Entity(entity)
	}
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
