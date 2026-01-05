mod spawn_new_skill;
mod spawn_points_definition;

use crate::components::skill_prefabs::{
	skill_contact::SkillContact,
	skill_projection::SkillProjection,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	components::{child_of_persistent::ChildOfPersistent, persistent_entity::PersistentEntity},
	traits::{
		accessors::get::{GetContextMut, GetMut},
		handles_skill_behaviors::{NewSkill, SkillEntities, SkillRoot},
		handles_skill_spawning::SkillSpawnPoints,
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct SkillSpawnerMut<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
}

impl GetContextMut<SkillSpawnPoints> for SkillSpawnerMut<'_, '_> {
	type TContext<'ctx> = SpawnPointContextMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SkillSpawnerMut,
		SkillSpawnPoints { entity }: SkillSpawnPoints,
	) -> Option<Self::TContext<'ctx>> {
		let entity = param.commands.get_mut(&entity)?;

		Some(SpawnPointContextMut { entity })
	}
}

pub struct SpawnPointContextMut<'ctx> {
	entity: ZyheedaEntityCommands<'ctx>,
}

impl<'w, 's> GetContextMut<NewSkill> for SkillSpawnerMut<'w, 's> {
	type TContext<'ctx> = SpawnNewSkillContextMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SkillSpawnerMut,
		_: NewSkill,
	) -> Option<Self::TContext<'ctx>> {
		Some(SpawnNewSkillContextMut {
			spawn: Box::new(|contact, projection| {
				let persistent_entity = PersistentEntity::default();
				let contact = param.commands.spawn((contact, persistent_entity)).id();
				let projection = param
					.commands
					.spawn((projection, ChildOfPersistent(persistent_entity)))
					.id();

				SkillEntities {
					root: SkillRoot {
						entity: contact,
						persistent_entity,
					},
					contact,
					projection,
				}
			}),
		})
	}
}

pub struct SpawnNewSkillContextMut<'ctx> {
	spawn: SpawnSkillFn<'ctx>,
}

type SpawnSkillFn<'a> = Box<dyn FnMut(SkillContact, SkillProjection) -> SkillEntities + 'a>;
