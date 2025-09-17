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
	+ for<'w, 's, 'a> GetFromSystemParam<'w, 's, SlotKey, TItem<'a> = TItem>
	+ for<'w, 's, 'a> GetFromSystemParam<'w, 's, AvailableSkills<SlotKey>, TItem<'a> = TSkills>
{
}

impl<T, TItem, TSkills> SlotComponent<TItem, TSkills> for T where
	T: Component<Mutability = Mutable>
		+ LoadoutKey<TKey = SlotKey>
		+ LoadoutItem<TItem = TItem>
		+ SwapInternal
		+ for<'w, 's, 'a> GetFromSystemParam<'w, 's, SlotKey, TItem<'a> = TItem>
		+ for<'w, 's, 'a> GetFromSystemParam<'w, 's, AvailableSkills<SlotKey>, TItem<'a> = TSkills>
{
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AvailableSkills<T>(pub T);
