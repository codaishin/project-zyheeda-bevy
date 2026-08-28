pub mod beam;
pub mod ground_target;
pub mod projectile;
pub mod shield;

use crate::{
	components::persistent_entity::PersistentEntity,
	effects::{force::Force, gravity::Gravity, health_damage::HealthDamage},
	tools::{action_key::slot::SlotKey, bone_name::BoneName, scale::Scale},
	traits::{
		accessors::get::{TryGetContext, TryGetContextMut},
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
use macros::{EntityKey, serde_model};
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
	type TSkillSpawnerMut: for<'w, 's> SystemParam<Item<'w, 's>: SpawnSkill + DespawnSkill>;
}

pub trait SpawnSkill {
	fn spawn_skill(&mut self, args: SpawnArgs<'_>) -> PersistentEntity;
}

impl<T> SpawnSkill for T
where
	T: DerefMut<Target: SpawnSkill>,
{
	fn spawn_skill(&mut self, args: SpawnArgs<'_>) -> PersistentEntity {
		self.deref_mut().spawn_skill(args)
	}
}

pub trait DespawnSkill {
	fn despawn_skill(&mut self, skill: SkillEntity);
}

impl<T> DespawnSkill for T
where
	T: DerefMut<Target: DespawnSkill>,
{
	fn despawn_skill(&mut self, skill: SkillEntity) {
		self.deref_mut().despawn_skill(skill);
	}
}

pub trait HandlesPhysicalSkillAgent {
	type TAgent: SystemParam + for<'c> TryGetContext<InitializedAgent, TContext<'c>: Target>;
	type TAgentMut: SystemParam
		+ for<'c> TryGetContextMut<NotInitializedAgent, TContext<'c>: Initialize>
		+ for<'c> TryGetContextMut<InitializedAgent, TContext<'c>: TargetMut>;
}

#[derive(EntityKey)]
pub struct NotInitializedAgent {
	pub entity: Entity,
}

#[derive(EntityKey)]
pub struct InitializedAgent {
	pub entity: Entity,
}

pub trait Initialize {
	fn initialize(
		&mut self,
		definition: HashMap<BoneName, SkillMountBone>,
		self_skill_scale: Scale<3>,
	);
}

impl<T> Initialize for T
where
	T: DerefMut<Target: Initialize>,
{
	fn initialize(
		&mut self,
		definition: HashMap<BoneName, SkillMountBone>,
		self_skill_scale: Scale<3>,
	) {
		self.deref_mut().initialize(definition, self_skill_scale);
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
	pub contact_effects: &'a [SkillEffect],
	pub projection_effects: &'a [SkillEffect],
	pub caster: SkillCaster,
	pub mount: SkillMount,
}

#[serde_model]
#[derive(Debug, PartialEq, Clone)]
pub enum SkillShape {
	SphereAoE(SphereAoE),
	Projectile(Projectile),
	Beam(Beam),
	Shield(Shield),
}

#[serde_model]
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SkillEffect {
	Force(Force),
	Gravity(Gravity),
	HealthDamage(HealthDamage),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SkillRoot {
	pub entity: Entity,
	pub persistent_entity: PersistentEntity,
}

#[serde_model]
#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub struct SkillCaster(pub PersistentEntity);

impl Deref for SkillCaster {
	type Target = PersistentEntity;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[serde_model]
#[derive(Debug, PartialEq, Clone, Copy)]
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

#[serde_model]
#[derive(Debug, PartialEq, Default, Clone, Copy)]
pub enum Cursor {
	#[default]
	Direction,
	TerrainHover,
}

#[serde_model]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub enum SkillMount {
	#[default]
	Center,
	Bone(SkillMountBone),
}

impl SkillMount {
	pub const fn center() -> Self {
		Self::Center
	}

	pub const fn neutral_slot() -> Self {
		Self::Bone(SkillMountBone::NeutralSlot)
	}

	pub const fn slot(key: SlotKey) -> Self {
		Self::Bone(SkillMountBone::Slot(key))
	}
}

#[serde_model]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Default)]
pub enum SkillMountBone {
	#[default]
	NeutralSlot,
	Slot(SlotKey),
}
