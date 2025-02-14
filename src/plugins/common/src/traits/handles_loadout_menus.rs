use super::{accessors::get::Getter, thread_safe::ThreadSafe};
use crate::tools::{inventory_key::InventoryKey, slot_key::SlotKey};
use bevy::prelude::*;
use std::{collections::HashMap, hash::Hash};

pub trait HandlesLoadoutMenu {
	fn inventory_with_swapper<TSwap>() -> impl ConfigureInventory<TSwap>
	where
		TSwap: Component + SwapValuesByKey;

	fn configure_quickbar_menu<TCache, TSystemMarker>(
		app: &mut App,
		get_quickbar_cache: impl IntoSystem<(), Option<TCache>, TSystemMarker>,
	) where
		TCache: GetDescriptor<SlotKey, TItem = QuickbarDescriptor> + ThreadSafe;
}

pub trait ConfigureInventory<TSwap> {
	fn configure<TInventory, TSlots, TSystemMarker1, TSystemMarker2>(
		&self,
		app: &mut App,
		get_inventor_descriptors: impl IntoSystem<(), Option<TInventory>, TSystemMarker1>,
		get_slot_descriptors: impl IntoSystem<(), Option<TSlots>, TSystemMarker2>,
	) where
		TInventory: GetDescriptor<InventoryKey> + ThreadSafe,
		TInventory::TItem: Getter<Name>,
		TSlots: GetDescriptor<SlotKey> + ThreadSafe,
		TSlots::TItem: Getter<Name>;
}

pub trait SwapValuesByKey {
	fn swap(&mut self, a: SwapKey, b: SwapKey);
}

pub trait GetDescriptor<TKey> {
	type TItem;

	fn get_descriptor(&self, key: TKey) -> Option<&Self::TItem>;
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct InventoryDescriptor {
	pub name: String,
	pub icon: Option<Handle<Image>>,
}

impl Getter<Name> for InventoryDescriptor {
	fn get(&self) -> Name {
		Name::from(self.name.clone())
	}
}

#[derive(Debug, PartialEq, Default, Clone)]
pub struct QuickbarDescriptor {
	pub name: String,
	pub execution: SkillExecution,
	pub icon: Option<Handle<Image>>,
}

impl Getter<Name> for QuickbarDescriptor {
	fn get(&self) -> Name {
		Name::from(self.name.clone())
	}
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

// Needs to be moved to skills plugin
#[derive(Debug, PartialEq)]
pub struct Descriptions<TKey, TItem>(pub HashMap<TKey, TItem>)
where
	TKey: Eq + Hash;

impl<TKey, TItem> GetDescriptor<TKey> for Descriptions<TKey, TItem>
where
	TKey: Eq + Hash,
{
	type TItem = TItem;

	fn get_descriptor(&self, key: TKey) -> Option<&TItem> {
		self.0.get(&key)
	}
}
