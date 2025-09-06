use crate::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::GetParamEntry,
		handles_loadout::loadout::{LoadoutItem, LoadoutKey, SwapInternal},
	},
};
use bevy::{ecs::component::Mutable, prelude::*};

pub trait SlotComponent<TItemEntry, TSkills>:
	Component<Mutability = Mutable>
	+ LoadoutKey<TKey = SlotKey>
	+ LoadoutItem<TItem = TItemEntry>
	+ SwapInternal
	+ for<'w, 's> GetParamEntry<'w, 's, SlotKey, TEntry = TItemEntry>
	+ for<'w, 's> GetParamEntry<'w, 's, AvailableSkills<SlotKey>, TEntry = TSkills>
{
}

impl<T, TItemEntry, TSkills> SlotComponent<TItemEntry, TSkills> for T where
	T: Component<Mutability = Mutable>
		+ LoadoutKey<TKey = SlotKey>
		+ LoadoutItem<TItem = TItemEntry>
		+ SwapInternal
		+ for<'w, 's> GetParamEntry<'w, 's, SlotKey, TEntry = TItemEntry>
		+ for<'w, 's> GetParamEntry<'w, 's, AvailableSkills<SlotKey>, TEntry = TSkills>
{
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AvailableSkills<T>(pub T);
