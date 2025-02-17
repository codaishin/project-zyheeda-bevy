use super::{
	handles_combo_menu::{InspectAble, InspectMarker, SkillIcon},
	thread_safe::ThreadSafe,
};
use crate::tools::{inventory_key::InventoryKey, slot_key::SlotKey};
use bevy::prelude::*;
use std::hash::Hash;

pub trait HandlesLoadoutMenu {
	fn loadout_with_swapper<TSwap>() -> impl ConfigureInventory<TSwap>
	where
		TSwap: Component + SwapValuesByKey;

	fn configure_quickbar_menu<TContainer, TSystemMarker>(
		app: &mut App,
		get_quickbar_cache: impl IntoSystem<(), Option<TContainer>, TSystemMarker>,
	) where
		TContainer: GetItem<SlotKey> + ThreadSafe,
		TContainer::TItem:
			InspectAble<ItemDescription> + InspectAble<SkillIcon> + InspectAble<SkillExecution>;
}

pub trait ConfigureInventory<TSwap> {
	fn configure<TInventory, TSlots, TSystemMarker1, TSystemMarker2>(
		&self,
		app: &mut App,
		get_inventor_descriptors: impl IntoSystem<(), Option<TInventory>, TSystemMarker1>,
		get_slot_descriptors: impl IntoSystem<(), Option<TSlots>, TSystemMarker2>,
	) where
		TInventory: GetItem<InventoryKey> + ThreadSafe,
		TInventory::TItem: InspectAble<ItemDescription>,
		TSlots: GetItem<SlotKey> + ThreadSafe,
		TSlots::TItem: InspectAble<ItemDescription>;
}

pub trait SwapValuesByKey {
	fn swap(&mut self, a: SwapKey, b: SwapKey);
}

pub trait GetItem<TKey> {
	type TItem;

	fn get_item(&self, key: TKey) -> Option<&Self::TItem>;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SwapKey {
	Inventory(InventoryKey),
	Slot(SlotKey),
}

impl From<InventoryKey> for SwapKey {
	fn from(key: InventoryKey) -> Self {
		Self::Inventory(key)
	}
}

impl From<SlotKey> for SwapKey {
	fn from(key: SlotKey) -> Self {
		Self::Slot(key)
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Default, Clone, Copy)]
pub enum SkillExecution {
	#[default]
	None,
	Active,
	Queued,
}

impl InspectMarker for SkillExecution {
	type TFieldRef<'a> = &'a SkillExecution;
}

#[derive(Debug, PartialEq)]
pub struct ItemDescription;

impl InspectMarker for ItemDescription {
	type TFieldRef<'a> = String;
}
