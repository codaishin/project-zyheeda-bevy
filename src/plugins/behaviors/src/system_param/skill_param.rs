mod hold_skill;
mod spawn_points_definition;

use crate::components::skill_usage::SkillUsage;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, GetMut},
		handles_skills_control::{SkillControl, SkillSpawnPoints},
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct SkillParamMut<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
	skill_usages: Query<'w, 's, Mut<'static, SkillUsage>>,
}

impl GetContextMut<SkillControl> for SkillParamMut<'_, '_> {
	type TContext<'ctx> = SkillContextMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SkillParamMut,
		SkillControl { entity }: SkillControl,
	) -> Option<Self::TContext<'ctx>> {
		if let Ok(skill_usage) = param.skill_usages.get_mut(entity) {
			return Some(SkillContextMut::Mut(skill_usage));
		}
		let entity = param.commands.get_mut(&entity)?;

		Some(SkillContextMut::New {
			entity,
			usage: SkillUsage::default(),
		})
	}
}

impl GetContextMut<SkillSpawnPoints> for SkillParamMut<'_, '_> {
	type TContext<'ctx> = SpawnPointContextMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SkillParamMut,
		SkillSpawnPoints { entity }: SkillSpawnPoints,
	) -> Option<Self::TContext<'ctx>> {
		let entity = param.commands.get_mut(&entity)?;

		Some(SpawnPointContextMut { entity })
	}
}

pub enum SkillContextMut<'ctx> {
	Mut(Mut<'ctx, SkillUsage>),
	New {
		entity: ZyheedaEntityCommands<'ctx>,
		usage: SkillUsage,
	},
}

// FIXME: This is a bit unstable. Should a system call `get_entity_context_mut`
//        multiple times, it might not see the result of using earlier contexts, effectively
//        overriding their `SkillUsage` insertions.
//        But this is a temporary implementation until agents call the skills
//        plugin directly for skill usage, where we can use a more stable implementation, because
//        the loadout can be accessed directly
impl Drop for SkillContextMut<'_> {
	fn drop(&mut self) {
		let Self::New { entity, usage } = self else {
			return;
		};
		let mut new = SkillUsage::default();
		std::mem::swap(&mut new, usage);
		entity.try_insert(new);
	}
}

pub struct SpawnPointContextMut<'ctx> {
	entity: ZyheedaEntityCommands<'ctx>,
}
