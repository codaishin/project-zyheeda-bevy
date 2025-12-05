mod read;
mod write;

use crate::components::{
	held_slots::{Current, HeldSlots},
	queue::Queue,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::{
	accessors::get::{ContextChanged, GetContext, GetContextMut},
	handles_loadout::skills::Skills,
};

#[derive(SystemParam)]
pub struct LoadoutActivityReader<'w, 's> {
	queues: Query<'w, 's, Ref<'static, Queue>>,
	held_slots: Query<'w, 's, Ref<'static, HeldSlots<Current>>>,
}

impl GetContext<Skills> for LoadoutActivityReader<'_, '_> {
	type TContext<'ctx> = LoadoutActivityReadContext<'ctx>;

	fn get_context<'ctx>(
		param: &'ctx LoadoutActivityReader,
		Skills { entity }: Skills,
	) -> Option<Self::TContext<'ctx>> {
		let queue = param.queues.get(entity).ok()?;
		let held_slots = param.held_slots.get(entity).ok()?;

		Some(LoadoutActivityReadContext { queue, held_slots })
	}
}

pub struct LoadoutActivityReadContext<'a> {
	queue: Ref<'a, Queue>,
	held_slots: Ref<'a, HeldSlots<Current>>,
}

impl ContextChanged for LoadoutActivityReadContext<'_> {
	fn context_changed(&self) -> bool {
		self.queue.is_changed() || self.held_slots.is_changed()
	}
}

#[derive(SystemParam)]
pub struct LoadoutActivityWriter<'w, 's> {
	held_slots: Query<'w, 's, &'static mut HeldSlots<Current>>,
}

impl GetContextMut<Skills> for LoadoutActivityWriter<'_, '_> {
	type TContext<'ctx> = LoadoutActivityWriteContext<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut LoadoutActivityWriter,
		Skills { entity }: Skills,
	) -> Option<Self::TContext<'ctx>> {
		let held_slots = param.held_slots.get_mut(entity).ok()?;

		Some(LoadoutActivityWriteContext { held_slots })
	}
}

pub struct LoadoutActivityWriteContext<'a> {
	held_slots: Mut<'a, HeldSlots<Current>>,
}
