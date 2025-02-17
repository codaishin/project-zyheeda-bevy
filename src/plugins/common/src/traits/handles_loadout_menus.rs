use super::{
	handles_combo_menu::{InspectAble, InspectMarker, SkillIcon},
	thread_safe::ThreadSafe,
};
use crate::tools::{inventory_key::InventoryKey, slot_key::SlotKey};
use bevy::prelude::*;
use std::{collections::HashMap, hash::Hash};

pub trait HandlesLoadoutMenu {
	fn inventory_with_swapper<TSwap>() -> impl ConfigureInventory<TSwap>
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

// Needs to be moved to skills plugin
#[derive(Debug, PartialEq)]
pub struct Cache<TKey, TItem>(pub HashMap<TKey, TItem>)
where
	TKey: Eq + Hash;

impl<TKey, TItem> GetItem<TKey> for Cache<TKey, TItem>
where
	TKey: Eq + Hash,
{
	type TItem = TItem;

	fn get_item(&self, key: TKey) -> Option<&TItem> {
		self.0.get(&key)
	}
}

pub struct QuickbarItem {
	pub name: String,
	pub icon: Option<Handle<Image>>,
	pub execution: SkillExecution,
}

impl InspectAble<ItemDescription> for QuickbarItem {
	fn get_inspect_able_field(&self) -> String {
		self.name.clone()
	}
}

impl InspectAble<SkillIcon> for QuickbarItem {
	fn get_inspect_able_field(&self) -> &Option<Handle<Image>> {
		&self.icon
	}
}

impl InspectAble<SkillExecution> for QuickbarItem {
	fn get_inspect_able_field(&self) -> &SkillExecution {
		&self.execution
	}
}

pub struct InventoryItem {
	pub name: String,
	pub skill_icon: Option<Handle<Image>>,
}

impl InspectAble<ItemDescription> for InventoryItem {
	fn get_inspect_able_field(&self) -> String {
		self.name.clone()
	}
}

impl InspectAble<SkillIcon> for InventoryItem {
	fn get_inspect_able_field(&self) -> &Option<Handle<Image>> {
		&self.skill_icon
	}
}

#[derive(Debug, PartialEq)]
pub struct ItemDescription;

impl InspectMarker for ItemDescription {
	type TFieldRef<'a> = String;
}
