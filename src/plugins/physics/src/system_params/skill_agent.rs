mod despawn_skill;
mod initialize;
mod spawn_new_skill;
mod target;

use crate::components::{skill::Skill, target::Target};
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
	targets: Query<'w, 's, &'static mut Target>,
	commands: ZyheedaCommands<'w, 's>,
}

impl GetContextMut<NotInitializedAgent> for SkillAgentMut<'_, '_> {
	type TContext<'ctx> = SkillAgentInitializerContext<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SkillAgentMut,
		NotInitializedAgent { entity }: NotInitializedAgent,
	) -> Option<Self::TContext<'ctx>> {
		if param.targets.contains(entity) {
			return None;
		}

		let entity = param.commands.get_mut(&entity)?;

		Some(SkillAgentInitializerContext { entity })
	}
}

impl GetContextMut<InitializedAgent> for SkillAgentMut<'_, '_> {
	type TContext<'ctx> = SkillAgentContextMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SkillAgentMut,
		InitializedAgent { entity }: InitializedAgent,
	) -> Option<Self::TContext<'ctx>> {
		let target = param.targets.get_mut(entity).ok()?;

		Some(SkillAgentContextMut { target })
	}
}

pub struct SkillAgentInitializerContext<'ctx> {
	entity: ZyheedaEntityCommands<'ctx>,
}

pub struct SkillAgentContextMut<'ctx> {
	target: Mut<'ctx, Target>,
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn no_uninitialized_context_when_target_present() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn(Target(None)).id();

		let ctx = app
			.world_mut()
			.run_system_once(move |mut p: SkillAgentMut| {
				SkillAgentMut::get_context_mut(&mut p, NotInitializedAgent { entity }).is_some()
			})?;

		assert!(!ctx);
		Ok(())
	}

	#[test]
	fn no_initialized_context_when_mount_points_missing() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();

		let ctx = app
			.world_mut()
			.run_system_once(move |mut p: SkillAgentMut| {
				SkillAgentMut::get_context_mut(&mut p, InitializedAgent { entity }).is_some()
			})?;

		assert!(!ctx);
		Ok(())
	}
}
