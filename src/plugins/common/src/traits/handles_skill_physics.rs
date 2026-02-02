pub mod beam;
pub mod ground_target;
pub mod projectile;
pub mod shield;

use crate::{
	components::persistent_entity::PersistentEntity,
	effects::{force::Force, gravity::Gravity, health_damage::HealthDamage},
	tools::{Index, action_key::slot::SlotKey, bone_name::BoneName},
	traits::{
		accessors::get::GetContextMut,
		handles_skill_physics::{
			beam::Beam,
			ground_target::SphereAoE,
			projectile::Projectile,
			shield::Shield,
		},
	},
};
use bevy::{
	ecs::{entity::Entity, system::SystemParam},
	prelude::*,
};
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
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
}

pub type SkillSpawnerMut<'w, 's, T> = <T as HandlesNewPhysicalSkill>::TSkillSpawnerMut<'w, 's>;

pub trait Spawn {
	fn spawn(&mut self, args: SpawnArgs<'_>) -> PersistentEntity;
}

impl<T> Spawn for T
where
	T: DerefMut<Target: Spawn>,
{
	fn spawn(&mut self, args: SpawnArgs<'_>) -> PersistentEntity {
		self.deref_mut().spawn(args)
	}
}

pub trait Despawn {
	fn despawn(&mut self, skill: SkillEntity);
}

impl<T> Despawn for T
where
	T: DerefMut<Target: Despawn>,
{
	fn despawn(&mut self, skill: SkillEntity) {
		self.deref_mut().despawn(skill);
	}
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

#[derive(Debug, PartialEq)]
pub struct SkillEntity(pub PersistentEntity);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SpawnArgs<'a> {
	pub shape: &'a SkillShape,
	pub contact_effects: &'a [Effect],
	pub projection_effects: &'a [Effect],
	pub caster: SkillCaster,
	pub spawner: SkillSpawner,
	pub target: SkillTarget,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum SkillShape {
	SphereAoE(SphereAoE),
	Projectile(Projectile),
	Beam(Beam),
	Shield(Shield),
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub enum Effect {
	Force(Force),
	Gravity(Gravity),
	HealthDamage(HealthDamage),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SkillRoot {
	pub entity: Entity,
	pub persistent_entity: PersistentEntity,
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

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::platform::collections::HashSet;

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
