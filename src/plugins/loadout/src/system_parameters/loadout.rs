mod read;
pub(crate) mod write;

use crate::{
	components::{
		combos::CombosInternal,
		inventory::Inventory,
		queue::Queue,
		slot_definitions::SlotDefinitions,
		slots::Slots,
	},
	item::Item,
	skills::Skill,
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::prelude::*;

#[derive(SystemParam)]
pub struct LoadoutReader<'w, 's> {
	agents: Query<'w, 's, ReadComponents>,
	items: Res<'w, Assets<Item>>,
	skills: Res<'w, Assets<Skill>>,
}

type ReadComponents = (
	Ref<'static, Slots>,
	Ref<'static, Inventory>,
	Ref<'static, CombosInternal>,
	Ref<'static, Queue>,
);

#[derive(SystemParam)]
pub struct LoadoutWriter<'w, 's> {
	slots: Query<'w, 's, &'static mut Slots>,
	inventories: Query<'w, 's, &'static mut Inventory>,
	combos: Query<'w, 's, &'static mut CombosInternal>,
	skills: Res<'w, Assets<Skill>>,
}

#[derive(SystemParam)]
pub struct LoadoutPrep<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
	slots: Query<'w, 's, &'static mut Slots>,
	inventories: Query<'w, 's, &'static mut Inventory>,
	slot_definitions: Query<'w, 's, (), With<SlotDefinitions>>,
}
