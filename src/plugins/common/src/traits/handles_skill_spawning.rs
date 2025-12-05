use crate::{
	tools::bone_name::BoneName,
	traits::{accessors::get::GetContextMut, handles_skill_behaviors::SkillSpawner},
};
use bevy::ecs::{entity::Entity, system::SystemParam};
use std::{collections::HashMap, ops::DerefMut};

pub trait HandlesSkillSpawning {
	type TSkillSpawnerMut<'w, 's>: SystemParam
		+ for<'c> GetContextMut<SkillSpawnPoints, TContext<'c>: SpawnPointsDefinition>;
}

pub type SkillSpawnerMut<'w, 's, T> = <T as HandlesSkillSpawning>::TSkillSpawnerMut<'w, 's>;

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
