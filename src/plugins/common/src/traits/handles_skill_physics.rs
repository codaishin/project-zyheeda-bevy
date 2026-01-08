use crate::{
	components::{asset_model::AssetModel, persistent_entity::PersistentEntity},
	effects::{force::Force, gravity::Gravity, health_damage::HealthDamage},
	tools::{Index, Units, UnitsPerSecond, action_key::slot::SlotKey, bone_name::BoneName},
	traits::{
		accessors::get::GetContextMut,
		handles_physics::colliders::{Blocker, Shape},
	},
	zyheeda_commands::ZyheedaCommands,
};
use bevy::{
	ecs::{entity::Entity, system::SystemParam},
	prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{
	collections::{HashMap, HashSet},
	ops::{Deref, DerefMut},
};

pub trait HandlesSkillPhysics:
	HandlesNewPhysicalSkill + HandlesPhysicalSkillSpawnPoints + HandlesPhysicalSkillComponents
{
}

impl<T> HandlesSkillPhysics for T where
	T: HandlesNewPhysicalSkill + HandlesPhysicalSkillSpawnPoints + HandlesPhysicalSkillComponents
{
}

pub trait HandlesPhysicalSkillComponents {
	type TSkillContact: Component;
	type TSkillProjection: Component;
}

pub trait HandlesNewPhysicalSkill {
	/// Skill spawner
	///
	/// Implementations of this are likely to use [`Commands`]. Insertion of skill components/effects
	/// and despawning should be handled through this [`SystemParam`].
	type TSkillSpawnerMut<'world, 'state>: for<'w, 's> SystemParam<Item<'w, 's>: Spawn + Despawn>;

	/// Skills always have a contact and a projection shape.
	///
	/// Activity of those shapes should be controlled by their attached effects.
	fn spawn_skill(
		commands: &mut ZyheedaCommands,
		contact: Contact,
		projection: Projection,
	) -> SkillEntities;
}

pub type SkillSpawnerMut<'w, 's, T> = <T as HandlesNewPhysicalSkill>::TSkillSpawnerMut<'w, 's>;

pub trait Spawn {
	type TSkill<'c>: Skill
	where
		Self: 'c;

	fn spawn(&mut self, contact: Contact, projection: Projection) -> Self::TSkill<'_>;
}

pub trait Despawn {
	fn despawn(&mut self, skill: SkillEntity);
}

pub trait HandlesPhysicalSkillSpawnPoints {
	type TSkillSpawnPointsMut<'w, 's>: SystemParam
		+ for<'c> GetContextMut<SkillSpawnPoints, TContext<'c>: RegisterDefinition>;
}

pub type SkillSpawnPointsMut<'w, 's, T> =
	<T as HandlesPhysicalSkillSpawnPoints>::TSkillSpawnPointsMut<'w, 's>;

pub struct SkillSpawnPoints {
	pub entity: Entity,
}

impl From<SkillSpawnPoints> for Entity {
	fn from(SkillSpawnPoints { entity }: SkillSpawnPoints) -> Self {
		entity
	}
}

pub trait RegisterDefinition {
	fn register_definition(&mut self, definition: HashMap<BoneName, SkillSpawner>);
}

impl<T> RegisterDefinition for T
where
	T: DerefMut<Target: RegisterDefinition>,
{
	fn register_definition(&mut self, definition: HashMap<BoneName, SkillSpawner>) {
		self.deref_mut().register_definition(definition);
	}
}

pub struct SkillEntity(pub PersistentEntity);

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Effect {
	Force(Force),
	Gravity(Gravity),
	HealthDamage(HealthDamage),
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

/// A newly spawned skill.
///
/// The provided insertion methods can be used to avoid usage of [`Commands`], which might conflict
/// with the [`SystemParam`] used to spawn the skill.
pub trait Skill {
	fn root(&self) -> PersistentEntity;
	fn insert_on_root<T>(&mut self, bundle: T)
	where
		T: Bundle;
	fn insert_on_contact(&mut self, effect: Effect);
	fn insert_on_projection(&mut self, effect: Effect);
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
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
