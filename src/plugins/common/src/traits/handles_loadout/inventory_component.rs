use crate::{
	tools::inventory_key::InventoryKey,
	traits::{
		accessors::get::GetFromSystemParam,
		handles_loadout::loadout::{LoadoutItem, LoadoutKey, SwapInternal},
	},
};
use bevy::{ecs::component::Mutable, prelude::*};

pub trait InventoryComponent<TItem>:
	Component<Mutability = Mutable>
	+ LoadoutKey<TKey = InventoryKey>
	+ LoadoutItem<TItem = TItem>
	+ SwapInternal
	+ for<'i> GetFromSystemParam<InventoryKey, TItem<'i> = TItem>
{
}

impl<T, TItem> InventoryComponent<TItem> for T where
	T: Component<Mutability = Mutable>
		+ LoadoutKey<TKey = InventoryKey>
		+ LoadoutItem<TItem = TItem>
		+ SwapInternal
		+ for<'i> GetFromSystemParam<InventoryKey, TItem<'i> = TItem>
{
}
