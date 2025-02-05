use super::{
	accessors::get::{Getter, GetterRef},
	thread_safe::ThreadSafe,
};
use crate::tools::{inventory_key::InventoryKey, item_type::ItemType, slot_key::SlotKey};
use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

pub trait HandlesEquipment {
	type TItem: Asset + Getter<ItemName> + Getter<ItemType>;
	type TInventory: Component
		+ ContinuousAccessMut<TItemHandle = Handle<Self::TItem>>
		+ SingleAccess<TItem = Self::TItem, TKey = InventoryKey>;

	type TSlots: Component + SingleAccess<TItem = Self::TItem, TKey = SlotKey>;

	type TSkill: Asset
		+ GetterRef<Option<Handle<Image>>>
		+ GetterRef<CompatibleItems>
		+ Getter<SkillDescription>
		+ Clone
		+ PartialEq
		+ ThreadSafe;

	type TCombos: Component
		+ GetFollowupKeys<TKey = SlotKey>
		+ GetCombosOrdered<Self::TSkill>
		+ UpdateConfig<Vec<SlotKey>, Option<Self::TSkill>>
		+ UpdateConfig<Vec<SlotKey>, SlotKey>;
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ItemName(pub String);

#[derive(Debug, PartialEq, Clone, Default)]
pub struct SkillDescription(pub String);

#[derive(Debug, PartialEq, Clone, Default)]
pub struct CompatibleItems(pub HashSet<ItemType>);

pub trait ContinuousAccessMut {
	type TItemHandle: Clone;

	fn continuous_access_mut(&mut self) -> &mut Vec<Option<Self::TItemHandle>>;
}

pub trait SingleAccess {
	type TKey;
	type TItem: Asset;

	fn single_access(&self, key: &Self::TKey) -> Option<&Handle<Self::TItem>>;
}

pub trait UpdateConfig<TKey, TValue> {
	fn update_config(&mut self, key: &TKey, value: TValue);
}

pub trait GetFollowupKeys {
	type TKey;

	fn followup_keys<T>(&self, after: T) -> Option<Vec<Self::TKey>>
	where
		T: Into<VecDeque<Self::TKey>> + 'static;
}

pub type Combo<'a, TSkill> = Vec<(Vec<SlotKey>, &'a TSkill)>;

pub trait GetCombosOrdered<TSkill> {
	fn combos_ordered<'a>(&'a self) -> impl Iterator<Item = Combo<'a, TSkill>>
	where
		TSkill: 'a;
}
