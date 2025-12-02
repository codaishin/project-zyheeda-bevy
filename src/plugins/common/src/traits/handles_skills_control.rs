use crate::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::GetContextMut,
		handles_animations::BoneName,
		handles_skill_behaviors::SkillSpawner,
	},
};
use bevy::ecs::{entity::Entity, system::SystemParam};
use std::{collections::HashMap, ops::DerefMut};

pub trait HandlesSkillControl {
	type TSkillControlMut<'w, 's>: SystemParam
		+ for<'c> GetContextMut<SkillControl, TContext<'c>: HoldSkill>
		+ for<'c> GetContextMut<SkillSpawnPoints, TContext<'c>: SpawnPointsDefinition>;
}

pub type SkillControlParamMut<'w, 's, T> = <T as HandlesSkillControl>::TSkillControlMut<'w, 's>;

pub struct SkillControl {
	pub entity: Entity,
}

impl From<SkillControl> for Entity {
	fn from(SkillControl { entity }: SkillControl) -> Self {
		entity
	}
}

pub trait HoldSkill {
	/// Set this each frame
	fn holding<TSlot>(&mut self, key: TSlot)
	where
		TSlot: Into<SlotKey> + 'static;
}

impl<T> HoldSkill for T
where
	T: DerefMut<Target: HoldSkill>,
{
	fn holding<TSlot>(&mut self, key: TSlot)
	where
		TSlot: Into<SlotKey> + 'static,
	{
		self.deref_mut().holding(key);
	}
}

pub struct SkillSpawnPoints {
	pub entity: Entity,
}

impl From<SkillSpawnPoints> for Entity {
	fn from(SkillSpawnPoints { entity }: SkillSpawnPoints) -> Self {
		entity
	}
}

pub trait SpawnPointsDefinition {
	fn insert_spawn_point_definition(&mut self, definition: HashMap<BoneName, SkillSpawner>);
}

impl<T> SpawnPointsDefinition for T
where
	T: DerefMut<Target: SpawnPointsDefinition>,
{
	fn insert_spawn_point_definition(&mut self, definition: HashMap<BoneName, SkillSpawner>) {
		self.deref_mut().insert_spawn_point_definition(definition);
	}
}
