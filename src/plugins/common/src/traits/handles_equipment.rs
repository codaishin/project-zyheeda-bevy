use super::{
	accessors::get::{Getter, GetterRef},
	thread_safe::ThreadSafe,
};
use crate::tools::{inventory_key::InventoryKey, item_type::ItemType, slot_key::SlotKey};
use bevy::prelude::*;
use std::collections::{HashSet, VecDeque};

/* FIXME:
 * This trait's design is overly complex due to the recent decoupling of the
 * skills and menu plugins. While it serves as a practical workaround for now,
 * a more structured and maintainable approach should be considered in the future.
 * Refactoring this will likely require significant changes to both plugins.
 */
pub trait HandlesEquipment {
	type TItem: Asset
		+ Getter<ItemName>
		+ Getter<ItemType>
		+ GetterRef<Option<Handle<Self::TSkill>>>;
	type TSkill: Asset
		+ GetterRef<Option<Handle<Image>>>
		+ GetterRef<CompatibleItems>
		+ Getter<SkillDescription>
		+ Clone
		+ PartialEq
		+ ThreadSafe;
	type TQueuedSkill: Getter<SlotKey> + GetterRef<Option<Handle<Image>>>;

	type TInventory: Component
		+ ItemAssetBufferMut<TItemHandle = Handle<Self::TItem>>
		+ ItemAsset<TItem = Self::TItem, TKey = InventoryKey>;
	type TSlots: Component
		+ ItemAsset<TItem = Self::TItem, TKey = SlotKey>
		+ WriteItem<SlotKey, Option<Handle<Self::TItem>>>;
	type TQueue: Component + IterateQueue<TItem = Self::TQueuedSkill>;
	type TCombos: Component
		+ FollowupKeys<TKey = SlotKey>
		+ GetCombosOrdered<Self::TSkill>
		+ WriteItem<Vec<SlotKey>, Option<Self::TSkill>>
		+ WriteItem<Vec<SlotKey>, SlotKey>
		+ PeekNext<TNext = Self::TSkill>;

	type TCombosTimeOut: Component + IsTimedOut;
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct CompatibleItems(pub HashSet<ItemType>);

pub trait ItemAssetBufferMut {
	type TItemHandle: Clone;

	fn buffer_mut(&mut self) -> &mut Vec<Option<Self::TItemHandle>>;
}

pub trait ItemAsset {
	type TKey;
	type TItem: Asset;

	fn item_asset(&self, key: &Self::TKey) -> Result<&Option<Handle<Self::TItem>>, KeyOutOfBounds>;
}

pub struct KeyOutOfBounds;

pub trait WriteItem<TKey, TValue> {
	fn write_item(&mut self, key: &TKey, value: TValue);
}

pub trait FollowupKeys {
	type TKey;

	fn followup_keys<T>(&self, after: T) -> Option<Vec<Self::TKey>>
	where
		T: Into<VecDeque<Self::TKey>> + 'static;
}

pub trait GetCombosOrdered<TSkill> {
	fn combos_ordered<'a>(&'a self) -> impl Iterator<Item = Combo<'a, TSkill>>
	where
		TSkill: 'a;
}

pub type Combo<'a, TSkill> = Vec<(Vec<SlotKey>, &'a TSkill)>;

pub trait PeekNext {
	type TNext;

	fn peek_next(&self, trigger: &SlotKey, item_type: &ItemType) -> Option<Self::TNext>;
}

pub trait IterateQueue {
	type TItem;

	fn iterate(&self) -> impl Iterator<Item = &Self::TItem>;
}

pub trait IsTimedOut {
	fn is_timed_out(&self) -> bool;
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ItemName(pub String);

#[derive(Debug, PartialEq, Clone, Default)]
pub struct SkillDescription(pub String);
