use crate::{
	components::{mount_points::MountPointsDefinition, target::Target},
	system_params::skill_agent::SkillAgentInitializerContext,
};
use common::{
	tools::bone_name::BoneName,
	traits::handles_skill_physics::{Initialize, SkillMount},
};
use std::collections::HashMap;

impl Initialize for SkillAgentInitializerContext<'_> {
	fn initialize(&mut self, definition: HashMap<BoneName, SkillMount>) {
		self.entity
			.try_insert((MountPointsDefinition(definition), Target(None)));
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::{components::target::Target, system_params::skill_agent::SkillAgentMut};
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
	};
	use common::{
		tools::action_key::slot::SlotKey,
		traits::{accessors::get::GetContextMut, handles_skill_physics::NotInitializedAgent},
	};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn insert_fix_points_definition() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let map = HashMap::from([
			(BoneName::from("a"), SkillMount::Neutral),
			(BoneName::from("b"), SkillMount::Slot(SlotKey(42))),
		]);
		let map_clone = map.clone();

		app.world_mut()
			.run_system_once(move |mut p: SkillAgentMut| {
				let mut ctx =
					SkillAgentMut::get_context_mut(&mut p, NotInitializedAgent { entity }).unwrap();

				ctx.initialize(map_clone.clone());
			})?;

		assert_eq!(
			Some(&MountPointsDefinition(map)),
			app.world()
				.entity(entity)
				.get::<MountPointsDefinition<SkillMount>>(),
		);
		Ok(())
	}

	#[test]
	fn insert_target() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let map = HashMap::from([]);
		let map_clone = map.clone();

		app.world_mut()
			.run_system_once(move |mut p: SkillAgentMut| {
				let mut ctx =
					SkillAgentMut::get_context_mut(&mut p, NotInitializedAgent { entity }).unwrap();

				ctx.initialize(map_clone.clone());
			})?;

		assert_eq!(
			Some(&Target(None)),
			app.world().entity(entity).get::<Target>(),
		);
		Ok(())
	}
}
