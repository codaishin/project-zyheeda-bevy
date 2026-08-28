use crate::{
	components::{
		mount_points::MountPointsDefinition,
		self_skill_scale::SelfSkillScale,
		target::SkillTargetInternal,
	},
	system_params::skill_agent::SkillAgentInitializerContext,
};
use common::prelude::*;
use std::collections::HashMap;

impl Initialize for SkillAgentInitializerContext<'_> {
	fn initialize(
		&mut self,
		definition: HashMap<BoneName, SkillMountBone>,
		self_skill_scale: Scale<3>,
	) {
		self.entity.try_insert((
			MountPointsDefinition(definition),
			SkillTargetInternal(None),
			SelfSkillScale(self_skill_scale),
		));
	}
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use crate::system_params::skill_agent::SkillAgentMut;
	use bevy::{
		app::{App, Update},
		ecs::system::{RunSystemError, RunSystemOnce},
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
			(BoneName::from("a"), SkillMountBone::NeutralSlot),
			(BoneName::from("b"), SkillMountBone::Slot(SlotKey(42))),
		]);
		let map_clone = map.clone();

		app.world_mut()
			.run_system_once(move |mut p: SkillAgentMut| {
				let mut ctx =
					SkillAgentMut::try_get_context_mut(&mut p, NotInitializedAgent { entity })
						.unwrap();

				ctx.initialize(map_clone.clone(), Scale::default());
			})?;

		assert_eq!(
			Some(&MountPointsDefinition(map)),
			app.world()
				.entity(entity)
				.get::<MountPointsDefinition<SkillMountBone>>(),
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
					SkillAgentMut::try_get_context_mut(&mut p, NotInitializedAgent { entity })
						.unwrap();

				ctx.initialize(map_clone.clone(), Scale::default());
			})?;

		assert_eq!(
			Some(&SkillTargetInternal(None)),
			app.world().entity(entity).get::<SkillTargetInternal>(),
		);
		Ok(())
	}

	#[test]
	fn insert_scale() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let map = HashMap::from([]);
		let map_clone = map.clone();

		app.world_mut()
			.run_system_once(move |mut p: SkillAgentMut| {
				let mut ctx =
					SkillAgentMut::try_get_context_mut(&mut p, NotInitializedAgent { entity })
						.unwrap();

				ctx.initialize(map_clone.clone(), scale!(1., 2., 3.));
			})?;

		assert_eq!(
			Some(&SelfSkillScale(scale!(1., 2., 3.))),
			app.world().entity(entity).get::<SelfSkillScale>(),
		);
		Ok(())
	}
}
