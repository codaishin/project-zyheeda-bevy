mod read;
mod write;

use crate::{components::queue::Queue, systems::enqueue::held_slots::HeldSlots};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::{
	accessors::get::{ContextChanged, TryGetContext, TryGetContextMut},
	handles_loadout::skills::Skills,
};
use zyheeda_core::prelude::*;

#[derive(SystemParam)]
pub struct LoadoutActivityReader<'w, 's> {
	loadout: Query<'w, 's, (&'static Queue, &'static HeldSlots)>,
}

impl TryGetContext<Skills> for LoadoutActivityReader<'static, 'static> {
	type TContext<'ctx> = LoadoutActivityReadContext<'ctx>;

	fn try_get_context<'ctx>(
		param: &'ctx LoadoutActivityReader,
		Skills { entity }: Skills,
	) -> Option<Self::TContext<'ctx>> {
		let (queue, held_slots) = param.loadout.get(entity).ok()?;

		Some(LoadoutActivityReadContext { queue, held_slots })
	}
}

pub struct LoadoutActivityReadContext<'a> {
	queue: &'a Queue,
	held_slots: &'a HeldSlots,
}

impl ContextChanged for LoadoutActivityReadContext<'_> {
	fn context_changed(&self) -> bool {
		any!(changed_this_frame(self.queue, self.held_slots))
	}
}

#[derive(SystemParam)]
pub struct LoadoutActivityWriter<'w, 's> {
	held_slots: Query<'w, 's, &'static mut HeldSlots>,
}

impl TryGetContextMut<Skills> for LoadoutActivityWriter<'static, 'static> {
	type TContext<'ctx> = LoadoutActivityWriteContext<'ctx>;

	fn try_get_context_mut<'ctx>(
		param: &'ctx mut LoadoutActivityWriter,
		Skills { entity }: Skills,
	) -> Option<Self::TContext<'ctx>> {
		let held_slots = param.held_slots.get_mut(entity).ok()?;

		Some(LoadoutActivityWriteContext { held_slots })
	}
}

pub struct LoadoutActivityWriteContext<'a> {
	held_slots: Mut<'a, HeldSlots>,
}

#[cfg(test)]
mod tests {
	#![allow(clippy::unwrap_used)]
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::tools::action_key::slot::SlotKey;
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn ctx_not_changed() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((Queue::default(), HeldSlots::default()))
			.id();

		let changed = app
			.world_mut()
			.run_system_once(move |r: LoadoutActivityReader| {
				let ctx = LoadoutActivityReader::try_get_context(&r, Skills { entity }).unwrap();
				ctx.context_changed()
			})?;

		assert!(!changed);
		Ok(())
	}

	#[test]
	fn ctx_changed_when_queue_changed() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((Queue::default().changed(), HeldSlots::default()))
			.id();

		let changed = app
			.world_mut()
			.run_system_once(move |r: LoadoutActivityReader| {
				let ctx = LoadoutActivityReader::try_get_context(&r, Skills { entity }).unwrap();
				ctx.context_changed()
			})?;

		assert!(changed);
		Ok(())
	}

	#[test]
	fn ctx_changed_when_held_slots_changed() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				Queue::default(),
				HeldSlots::default()
					.with_previous([SlotKey(11)])
					.with_current([SlotKey(11), SlotKey(22)]),
			))
			.id();

		let changed = app
			.world_mut()
			.run_system_once(move |r: LoadoutActivityReader| {
				let ctx = LoadoutActivityReader::try_get_context(&r, Skills { entity }).unwrap();
				ctx.context_changed()
			})?;

		assert!(changed);
		Ok(())
	}
}
