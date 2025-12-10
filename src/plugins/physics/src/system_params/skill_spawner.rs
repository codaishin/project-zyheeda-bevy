mod spawn_points_definition;

use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, GetMut},
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
