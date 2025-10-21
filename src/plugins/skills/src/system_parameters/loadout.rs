mod read;
mod write;

use crate::{
	components::{combos::Combos, inventory::Inventory, queue::Queue, slots::Slots},
	item::Item,
	skills::Skill,
};
use bevy::{ecs::system::SystemParam, prelude::*};

#[derive(SystemParam)]
pub struct LoadoutReader<'w, 's> {
	agents: Query<'w, 's, ReadComponents>,
	items: Res<'w, Assets<Item>>,
	skills: Res<'w, Assets<Skill>>,
}

type ReadComponents = (
	Ref<'static, Slots>,
	Ref<'static, Inventory>,
	Ref<'static, Combos>,
	Ref<'static, Queue>,
);

#[derive(SystemParam)]
pub struct LoadoutWriter<'w, 's> {
	agents: Query<'w, 's, WriteComponents>,
	skills: Res<'w, Assets<Skill>>,
}

type WriteComponents = (
	&'static mut Slots,
	&'static mut Inventory,
	&'static mut Combos,
);
