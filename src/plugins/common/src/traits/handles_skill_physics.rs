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
use macros::EntityKey;
use serde::{Deserialize, Serialize};
use std::{
	collections::HashMap,
	ops::{Deref, DerefMut},
};

pub trait HandlesSkillPhysics:
	HandlesNewPhysicalSkill + HandlesPhysicalSkillAgent + HandlesPhysicalSkillComponents
{
}

impl<T> HandlesSkillPhysics for T where
	T: HandlesNewPhysicalSkill + HandlesPhysicalSkillAgent + HandlesPhysicalSkillComponents
{
}

pub trait HandlesPhysicalSkillComponents {
	type TSkillContact: Component;
	type TSkillProjection: Component;
}

pub trait HandlesNewPhysicalSkill {
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

pub trait HandlesPhysicalSkillAgent {
	type TAgentMut<'w, 's>: SystemParam
		+ for<'c> GetContextMut<NotInitializedAgent, TContext<'c>: Initialize>
		+ for<'c> GetContextMut<InitializedAgent, TContext<'c>: TargetMut>;
}

pub type SkillAgentMut<'w, 's, T> = <T as HandlesPhysicalSkillAgent>::TAgentMut<'w, 's>;

#[derive(EntityKey)]
pub struct NotInitializedAgent {
	pub entity: Entity,
}

#[derive(EntityKey)]
pub struct InitializedAgent {
	pub entity: Entity,
}

pub trait Initialize {
	fn initialize(&mut self, definition: HashMap<BoneName, SkillMount>);
}

impl<T> Initialize for T
where
	T: DerefMut<Target: Initialize>,
{
	fn initialize(&mut self, definition: HashMap<BoneName, SkillMount>) {
		self.deref_mut().initialize(definition);
	}
}

pub trait Target {
	fn target(&self) -> Option<&SkillTarget>;
}

impl<T> Target for T
where
	T: Deref<Target: Target>,
{
	fn target(&self) -> Option<&SkillTarget> {
		self.deref().target()
	}
}

pub trait TargetMut: Target {
	fn target_mut(&mut self) -> &mut Option<SkillTarget>;
}

impl<T> TargetMut for T
where
	T: DerefMut<Target: TargetMut>,
{
	fn target_mut(&mut self) -> &mut Option<SkillTarget> {
		self.deref_mut().target_mut()
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
	pub mount: SkillMount,
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
	Cursor(Cursor),
	Entity(PersistentEntity),
}

impl Default for SkillTarget {
	fn default() -> Self {
		Self::Cursor(Cursor::default())
	}
}

impl From<PersistentEntity> for SkillTarget {
	fn from(entity: PersistentEntity) -> Self {
		Self::Entity(entity)
	}
}

#[derive(Debug, PartialEq, Default, Clone, Copy, Serialize, Deserialize)]
pub enum Cursor {
	#[default]
	Direction,
	TerrainHover,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default, Serialize, Deserialize)]
pub enum SkillMount {
	#[default]
	Neutral,
	Slot(SlotKey),
}

impl From<SkillMount> for Index<usize> {
	fn from(value: SkillMount) -> Self {
		match value {
			SkillMount::Neutral => Index(0),
			SkillMount::Slot(SlotKey(slot)) => Index(slot as usize + 1),
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
			SkillMount::Neutral,
			SkillMount::Slot(SlotKey(0)),
			SkillMount::Slot(SlotKey(42)),
			SkillMount::Slot(SlotKey(255)),
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
