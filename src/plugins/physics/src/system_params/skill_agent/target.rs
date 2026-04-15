use crate::system_params::skill_agent::SkillAgentContextMut;
use common::traits::handles_skill_physics::{SkillTarget, Target, TargetMut};

impl Target for SkillAgentContextMut<'_> {
	fn target(&self) -> Option<&SkillTarget> {
		self.target.0.as_ref()
	}
}

impl TargetMut for SkillAgentContextMut<'_> {
	fn target_mut(&mut self) -> &mut Option<SkillTarget> {
		&mut self.target.0
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{
		components::{mount_points::MountPointsDefinition, target::Target},
		system_params::skill_agent::SkillAgentMut,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::{
		components::persistent_entity::PersistentEntity,
		traits::{
			accessors::get::GetContextMut,
			handles_skill_physics::{InitializedAgent, SkillMount, Target as _},
		},
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn get_target() -> Result<(), RunSystemError> {
		let mut app = setup();
		let target_entity = PersistentEntity::default();
		let entity = app
			.world_mut()
			.spawn((
				Target(Some(SkillTarget::Entity(target_entity))),
				MountPointsDefinition::<SkillMount>::default(),
			))
			.id();

		let target = app
			.world_mut()
			.run_system_once(move |mut p: SkillAgentMut| {
				let key = InitializedAgent { entity };
				let ctx = SkillAgentMut::get_context_mut(&mut p, key).unwrap();
				ctx.target().copied()
			})?;

		assert_eq!(Some(SkillTarget::Entity(target_entity)), target);
		Ok(())
	}

	#[test]
	fn mut_target() -> Result<(), RunSystemError> {
		let mut app = setup();
		let target_entity = PersistentEntity::default();
		let entity = app
			.world_mut()
			.spawn((Target(None), MountPointsDefinition::<SkillMount>::default()))
			.id();

		app.world_mut()
			.run_system_once(move |mut p: SkillAgentMut| {
				let key = InitializedAgent { entity };
				let mut ctx = SkillAgentMut::get_context_mut(&mut p, key).unwrap();
				*ctx.target_mut() = Some(SkillTarget::Entity(target_entity));
			})?;

		assert_eq!(
			Some(SkillTarget::Entity(target_entity)),
			app.world()
				.entity(entity)
				.get::<Target>()
				.and_then(|Target(target)| *target)
		);
		Ok(())
	}
}
