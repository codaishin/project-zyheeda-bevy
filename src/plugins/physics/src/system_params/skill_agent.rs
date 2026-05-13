mod despawn_skill;
mod initialize;
mod spawn_new_skill;
mod target;

use crate::components::{mount_points::MountPointsDefinition, skill::Skill, target::Target};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::{
		accessors::get::{ContextChanged, GetContext, GetContextMut, GetMut},
		handles_skill_physics::{InitializedAgent, NotInitializedAgent, SkillMountBone},
	},
	zyheeda_commands::{ZyheedaCommands, ZyheedaEntityCommands},
};

#[derive(SystemParam)]
pub struct SkillAgent<'w, 's> {
	targets: Query<'w, 's, Ref<'static, Target>>,
}

impl GetContext<InitializedAgent> for SkillAgent<'static, 'static> {
	type TContext<'ctx> = SkillAgentContext<'ctx>;

	fn get_context<'ctx>(
		param: &'ctx SkillAgent,
		InitializedAgent { entity }: InitializedAgent,
	) -> Option<Self::TContext<'ctx>> {
		Some(SkillAgentContext {
			target: param.targets.get(entity).ok()?,
		})
	}
}

#[derive(SystemParam)]
pub struct SkillAgentMut<'w, 's> {
	skills: Query<'w, 's, (), With<Skill>>,
	mount_point_definitions: Query<'w, 's, (), With<MountPointsDefinition<SkillMountBone>>>,
	targets: Query<'w, 's, &'static mut Target>,
	commands: ZyheedaCommands<'w, 's>,
}

impl GetContextMut<NotInitializedAgent> for SkillAgentMut<'static, 'static> {
	type TContext<'ctx> = SkillAgentInitializerContext<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SkillAgentMut,
		NotInitializedAgent { entity }: NotInitializedAgent,
	) -> Option<Self::TContext<'ctx>> {
		if param.mount_point_definitions.contains(entity) && param.targets.contains(entity) {
			return None;
		}

		let entity = param.commands.get_mut(&entity)?;

		Some(SkillAgentInitializerContext { entity })
	}
}

impl GetContextMut<InitializedAgent> for SkillAgentMut<'static, 'static> {
	type TContext<'ctx> = SkillAgentContextMut<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut SkillAgentMut,
		InitializedAgent { entity }: InitializedAgent,
	) -> Option<Self::TContext<'ctx>> {
		if !param.mount_point_definitions.contains(entity) {
			return None;
		}

		let target = param.targets.get_mut(entity).ok()?;

		Some(SkillAgentContextMut { target })
	}
}

pub struct SkillAgentContext<'ctx> {
	target: Ref<'ctx, Target>,
}

impl ContextChanged for SkillAgentContext<'_> {
	fn context_changed(&self) -> bool {
		self.target.is_changed()
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
	use common::traits::handles_skill_physics::SkillMountBone;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	mod not_initialized {
		use super::*;

		#[test]
		fn get_context_when_target_missing() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(MountPointsDefinition::<SkillMountBone>::default())
				.id();

			let ctx = app
				.world_mut()
				.run_system_once(move |mut p: SkillAgentMut| {
					SkillAgentMut::get_context_mut(&mut p, NotInitializedAgent { entity }).is_some()
				})?;

			assert!(ctx);
			Ok(())
		}

		#[test]
		fn get_context_when_mount_points_missing() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn(Target(None)).id();

			let ctx = app
				.world_mut()
				.run_system_once(move |mut p: SkillAgentMut| {
					SkillAgentMut::get_context_mut(&mut p, NotInitializedAgent { entity }).is_some()
				})?;

			assert!(ctx);
			Ok(())
		}

		#[test]
		fn no_context_when_target_and_mount_points_present() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					MountPointsDefinition::<SkillMountBone>::default(),
					Target(None),
				))
				.id();

			let ctx = app
				.world_mut()
				.run_system_once(move |mut p: SkillAgentMut| {
					SkillAgentMut::get_context_mut(&mut p, NotInitializedAgent { entity }).is_some()
				})?;

			assert!(!ctx);
			Ok(())
		}
	}

	mod initialized {
		use super::*;

		#[test]
		fn no_context_when_target_missing() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn(MountPointsDefinition::<SkillMountBone>::default())
				.id();

			let ctx = app
				.world_mut()
				.run_system_once(move |mut p: SkillAgentMut| {
					SkillAgentMut::get_context_mut(&mut p, InitializedAgent { entity }).is_some()
				})?;

			assert!(!ctx);
			Ok(())
		}

		#[test]
		fn mo_context_when_mount_points_missing() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app.world_mut().spawn(Target(None)).id();

			let ctx = app
				.world_mut()
				.run_system_once(move |mut p: SkillAgentMut| {
					SkillAgentMut::get_context_mut(&mut p, InitializedAgent { entity }).is_some()
				})?;

			assert!(!ctx);
			Ok(())
		}

		#[test]
		fn get_context_when_target_and_mount_points_present() -> Result<(), RunSystemError> {
			let mut app = setup();
			let entity = app
				.world_mut()
				.spawn((
					MountPointsDefinition::<SkillMountBone>::default(),
					Target(None),
				))
				.id();

			let ctx = app
				.world_mut()
				.run_system_once(move |mut p: SkillAgentMut| {
					SkillAgentMut::get_context_mut(&mut p, InitializedAgent { entity }).is_some()
				})?;

			assert!(ctx);
			Ok(())
		}
	}
}
