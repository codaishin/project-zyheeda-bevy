use crate::system_parameters::loadout_activity::LoadoutActivityReadContext;
use common::{tools::action_key::slot::SlotKey, traits::handles_loadout::HeldSkills};
use std::collections::HashSet;

impl HeldSkills for LoadoutActivityReadContext<'_> {
	fn held_skills(&self) -> &HashSet<SlotKey> {
		&self.active_slots.slots
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		components::{
			active_slots::{ActiveSlots, Current},
			queue::Queue,
		},
		system_parameters::loadout_activity::LoadoutActivityReader,
	};
	use bevy::{
		ecs::system::{RunSystemError, RunSystemOnce},
		prelude::*,
	};
	use common::traits::{accessors::get::GetContext, handles_loadout::skills::Skills};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn return_held_skills() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				ActiveSlots::<Current>::from([SlotKey(0), SlotKey(1)]),
				Queue::default(),
			))
			.id();

		let active_skills = app
			.world_mut()
			.run_system_once(move |l: LoadoutActivityReader| {
				let ctx = LoadoutActivityReader::get_context(&l, Skills { entity }).unwrap();
				ctx.held_skills().clone()
			})?;

		assert_eq!(HashSet::from([SlotKey(0), SlotKey(1)]), active_skills);
		Ok(())
	}
}
