use crate::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::GetFromSystemParam,
		handles_loadout::loadout::{LoadoutItem, LoadoutKey, SwapInternal},
	},
};
use bevy::{ecs::component::Mutable, prelude::*};

pub trait SlotComponent<TItem, TSkills>:
	Component<Mutability = Mutable>
	+ LoadoutKey<TKey = SlotKey>
	+ LoadoutItem<TItem = TItem>
	+ SwapInternal
	+ for<'i> GetFromSystemParam<SlotKey, TItem<'i> = TItem>
	+ for<'i> GetFromSystemParam<AvailableSkills<SlotKey>, TItem<'i> = TSkills>
{
}

impl<T, TItem, TSkills> SlotComponent<TItem, TSkills> for T where
	T: Component<Mutability = Mutable>
		+ LoadoutKey<TKey = SlotKey>
		+ LoadoutItem<TItem = TItem>
		+ SwapInternal
		+ for<'i> GetFromSystemParam<SlotKey, TItem<'i> = TItem>
		+ for<'i> GetFromSystemParam<AvailableSkills<SlotKey>, TItem<'i> = TSkills>
{
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AvailableSkills<T>(pub T);
