use super::accessors::get::Getter;
use crate::tools::{inventory_key::InventoryKey, slot_key::SlotKey};
use bevy::prelude::*;

pub trait HandlesEquipment {
	type TItem: Asset + Getter<ItemName>;
	type TInventory: Component
		+ ContinuousAccessMut<TItem = Handle<Self::TItem>>
		+ SingleAccess<TItem = Self::TItem, TKey = InventoryKey>;

	type TSlots: Component + SingleAccess<TItem = Self::TItem, TKey = SlotKey>;
}

pub trait ContinuousAccessMut {
	type TItem: Clone;

	fn continuous_access_mut(&mut self) -> &mut Vec<Option<Self::TItem>>;
}

pub trait SingleAccess {
	type TKey;
	type TItem: Asset;

	fn single_access(&self, key: &Self::TKey) -> Option<&Handle<Self::TItem>>;
}

pub struct ItemName(pub String);
