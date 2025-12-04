use crate::system_parameters::loadout_activity::LoadoutActivityWriteContext;
use common::{
	tools::action_key::slot::SlotKey,
	traits::handles_loadout::{HeldSkills, HeldSkillsMut},
};
use std::collections::HashSet;

impl HeldSkills for LoadoutActivityWriteContext<'_> {
	fn held_skills(&self) -> &HashSet<SlotKey> {
		&self.active_slots.slots
	}
}

impl HeldSkillsMut for LoadoutActivityWriteContext<'_> {
	fn held_skills_mut(&mut self) -> &mut HashSet<SlotKey> {
		&mut self.active_slots.slots
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::active_slots::{ActiveSlots, Current},
		system_parameters::loadout_activity::LoadoutActivityWriter,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{accessors::get::GetContextMut, handles_loadout::skills::Skills};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn return_held_skills() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((ActiveSlots::<Current>::from([SlotKey(0), SlotKey(1)]),))
			.id();

		let active_skills =
			app.world_mut()
				.run_system_once(move |mut l: LoadoutActivityWriter| {
					let ctx =
						LoadoutActivityWriter::get_context_mut(&mut l, Skills { entity }).unwrap();
					ctx.held_skills().clone()
				})?;

		assert_eq!(HashSet::from([SlotKey(0), SlotKey(1)]), active_skills);
		Ok(())
	}

	#[test]
	fn mutate_held_skills() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((ActiveSlots::<Current>::from([SlotKey(0), SlotKey(1)]),))
			.id();

		app.world_mut()
			.run_system_once(move |mut l: LoadoutActivityWriter| {
				let mut ctx =
					LoadoutActivityWriter::get_context_mut(&mut l, Skills { entity }).unwrap();
				ctx.held_skills_mut().insert(SlotKey(2));
			})?;

		assert_eq!(
			Some(&ActiveSlots::from([SlotKey(0), SlotKey(1), SlotKey(2)])),
			app.world().entity(entity).get::<ActiveSlots<Current>>(),
		);
		Ok(())
	}
}
