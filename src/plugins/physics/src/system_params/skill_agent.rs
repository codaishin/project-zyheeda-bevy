mod despawn_skill;
mod initialize;
mod spawn_new_skill;
mod target;

use crate::components::skill::Skill;
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{GetContextMut, GetMut},
		handles_skill_physics::{InitializedAgent, NotInitializedAgent},
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct SkillAgentMut<'w, 's> {
	skills: Query<'w, 's, (), With<Skill>>,
	commands: ZyheedaCommands<'w, 's>,
}

impl GetContextMut<NotInitializedAgent> for SkillAgentMut<'_, '_> {
	type TContext<'ctx> = SkillAgentInitializerContext<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SkillAgentMut,
		NotInitializedAgent { entity }: NotInitializedAgent,
	) -> Option<Self::TContext<'ctx>> {
		let entity = param.commands.get_mut(&entity)?;

		Some(SkillAgentInitializerContext { entity })
	}
}

impl GetContextMut<InitializedAgent> for SkillAgentMut<'_, '_> {
	type TContext<'ctx> = SkillAgentContextMut;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SkillAgentMut,
		key: InitializedAgent,
	) -> Option<Self::TContext<'ctx>> {
		None
	}
}

pub struct SkillAgentInitializerContext<'ctx> {
	entity: ZyheedaEntityCommands<'ctx>,
}

pub struct SkillAgentContextMut {}
