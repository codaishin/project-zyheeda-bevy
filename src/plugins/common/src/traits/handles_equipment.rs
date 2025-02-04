use std::collections::VecDeque;

use super::accessors::get::Getter;
use crate::tools::{inventory_key::InventoryKey, item_type::ItemType, slot_key::SlotKey};
use bevy::prelude::*;

pub trait HandlesEquipment {
	type TItem: Asset + Getter<ItemName> + Getter<ItemType>;
	type TInventory: Component
		+ ContinuousAccessMut<TItemHandle = Handle<Self::TItem>>
		+ SingleAccess<TItem = Self::TItem, TKey = InventoryKey>;

	type TSlots: Component + SingleAccess<TItem = Self::TItem, TKey = SlotKey>;

	type TCombos: Component + GetFollowupKeys;
}

pub struct ItemName(pub String);

pub trait ContinuousAccessMut {
	type TItemHandle: Clone;

	fn continuous_access_mut(&mut self) -> &mut Vec<Option<Self::TItemHandle>>;
}

pub trait SingleAccess {
	type TKey;
	type TItem: Asset;

	fn single_access(&self, key: &Self::TKey) -> Option<&Handle<Self::TItem>>;
}

pub trait GetFollowupKeys {
	type TKey;

	fn followup_keys<T>(&self, after: T) -> Option<Vec<Self::TKey>>
	where
		T: Into<VecDeque<Self::TKey>>;
}
