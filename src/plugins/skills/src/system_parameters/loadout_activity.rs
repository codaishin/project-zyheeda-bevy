mod read;
mod write;

use crate::components::{
	active_slots::{ActiveSlots, Current},
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
	active_slots: Query<'w, 's, Ref<'static, ActiveSlots<Current>>>,
}

impl GetContext<Skills> for LoadoutActivityReader<'_, '_> {
	type TContext<'ctx> = LoadoutActivityReadContext<'ctx>;

	fn get_context<'ctx>(
		param: &'ctx LoadoutActivityReader,
		Skills { entity }: Skills,
	) -> Option<Self::TContext<'ctx>> {
		let queue = param.queues.get(entity).ok()?;
		let active_slots = param.active_slots.get(entity).ok()?;

		Some(LoadoutActivityReadContext {
			queue,
			active_slots,
		})
	}
}

pub struct LoadoutActivityReadContext<'a> {
	queue: Ref<'a, Queue>,
	active_slots: Ref<'a, ActiveSlots<Current>>,
}

impl ContextChanged for LoadoutActivityReadContext<'_> {
	fn context_changed(&self) -> bool {
		self.queue.is_changed() || self.active_slots.is_changed()
	}
}

#[derive(SystemParam)]
pub struct LoadoutActivityWriter<'w, 's> {
	active_slots: Query<'w, 's, &'static mut ActiveSlots<Current>>,
}

impl GetContextMut<Skills> for LoadoutActivityWriter<'_, '_> {
	type TContext<'ctx> = LoadoutActivityWriteContext<'ctx>;

	fn get_context_mut<'ctx>(
		param: &'ctx mut LoadoutActivityWriter,
		Skills { entity }: Skills,
	) -> Option<Self::TContext<'ctx>> {
		let active_slots = param.active_slots.get_mut(entity).ok()?;

		Some(LoadoutActivityWriteContext { active_slots })
	}
}

pub struct LoadoutActivityWriteContext<'a> {
	active_slots: Mut<'a, ActiveSlots<Current>>,
}
