use crate::{
	tools::action_key::slot::SlotKey,
	traits::handles_loadout::loadout::{LoadoutItem, LoadoutKey},
};
use bevy::{ecs::component::Mutable, prelude::*};
use std::collections::HashSet;

pub trait CombosComponent<TSkill>:
	Component<Mutability = Mutable>
	+ LoadoutKey<TKey = SlotKey>
	+ LoadoutItem<TItem = TSkill>
	+ GetCombosOrdered
	+ UpdateCombos
	+ NextConfiguredKeys<SlotKey>
{
}

impl<T, TSkill> CombosComponent<TSkill> for T where
	T: Component<Mutability = Mutable>
		+ LoadoutKey<TKey = SlotKey>
		+ LoadoutItem<TItem = TSkill>
		+ GetCombosOrdered
		+ UpdateCombos
		+ NextConfiguredKeys<SlotKey>
{
}

pub trait NextConfiguredKeys<TKey> {
	fn next_keys(&self, combo_keys: &[TKey]) -> HashSet<TKey>;
}

pub trait GetCombosOrdered: LoadoutKey + LoadoutItem {
	/// Get combos with a consistent ordering.
	/// The specific ordering heuristic is up to the implementor.
	fn combos_ordered(&self) -> Vec<Combo<Self::TKey, Self::TItem>>;
}

pub trait UpdateCombos: LoadoutKey + LoadoutItem {
	fn update_combos(&mut self, combos: Combo<Self::TKey, Option<Self::TItem>>);
}

pub type Combo<TKey, TSkill> = Vec<(Vec<TKey>, TSkill)>;
