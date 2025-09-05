use crate::{
	tools::inventory_key::InventoryKey,
	traits::{
		accessors::get::GetParamEntry,
		handles_loadout::loadout::{LoadoutItem, LoadoutKey, SwapInternal},
	},
};
use bevy::{ecs::component::Mutable, prelude::*};

pub trait InventoryComponent<TItemEntry>:
	Component<Mutability = Mutable>
	+ LoadoutKey<TKey = InventoryKey>
	+ LoadoutItem<TItem = TItemEntry>
	+ SwapInternal
	+ for<'w, 's> GetParamEntry<'w, 's, InventoryKey, TEntry = TItemEntry>
{
}

impl<T, TItemEntry> InventoryComponent<TItemEntry> for T where
	T: Component<Mutability = Mutable>
		+ LoadoutKey<TKey = InventoryKey>
		+ LoadoutItem<TItem = TItemEntry>
		+ SwapInternal
		+ for<'w, 's> GetParamEntry<'w, 's, InventoryKey, TEntry = TItemEntry>
{
}
