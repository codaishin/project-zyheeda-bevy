mod read;
mod write;

use crate::components::{held_slots::HeldSlots, queue::Queue};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::traits::{
	accessors::get::{ContextChanged, GetContext, GetContextMut},
	handles_loadout::skills::Skills,
};
use zyheeda_core::prelude::*;

#[derive(SystemParam)]
pub struct LoadoutActivityReader<'w, 's> {
	#[allow(clippy::type_complexity)]
	loadout: Query<'w, 's, (Ref<'static, Queue>, Ref<'static, HeldSlots>)>,
}

impl GetContext<Skills> for LoadoutActivityReader<'_, '_> {
	type TContext<'ctx> = LoadoutActivityReadContext<'ctx>;

	fn get_context<'ctx>(
		param: &'ctx LoadoutActivityReader,
		Skills { entity }: Skills,
	) -> Option<Self::TContext<'ctx>> {
		let (queue, held_slots) = param.loadout.get(entity).ok()?;

		Some(LoadoutActivityReadContext { queue, held_slots })
	}
}

pub struct LoadoutActivityReadContext<'a> {
	queue: Ref<'a, Queue>,
	held_slots: Ref<'a, HeldSlots>,
}

impl ContextChanged for LoadoutActivityReadContext<'_> {
	fn context_changed(&self) -> bool {
		any!(is_changed(self.queue, self.held_slots))
	}
}

#[derive(SystemParam)]
pub struct LoadoutActivityWriter<'w, 's> {
	held_slots: Query<'w, 's, &'static mut HeldSlots>,
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
	held_slots: Mut<'a, HeldSlots>,
}
