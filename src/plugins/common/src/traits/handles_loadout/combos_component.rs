use crate::{
	tools::action_key::slot::SlotKey,
	traits::handles_loadout::loadout::{LoadoutItem, LoadoutKey},
};
use bevy::{ecs::component::Mutable, prelude::*};
use std::{collections::HashSet, ops::Deref};

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

impl<T, TKey> NextConfiguredKeys<TKey> for T
where
	T: Deref<Target: NextConfiguredKeys<TKey>>,
{
	fn next_keys(&self, combo_keys: &[TKey]) -> HashSet<TKey> {
		self.deref().next_keys(combo_keys)
	}
}

pub trait GetCombosOrdered {
	type TSkill;

	/// Get combos with a consistent ordering.
	/// The specific ordering heuristic is up to the implementor.
	fn combos_ordered(&self) -> Vec<Combo<SlotKey, Self::TSkill>>;
}

impl<T> GetCombosOrdered for T
where
	T: Deref<Target: GetCombosOrdered>,
{
	type TSkill = <<T as Deref>::Target as GetCombosOrdered>::TSkill;

	fn combos_ordered(&self) -> Vec<Combo<SlotKey, Self::TSkill>> {
		self.deref().combos_ordered()
	}
}

pub trait UpdateCombos: LoadoutKey + LoadoutItem {
	fn update_combos(&mut self, combos: Combo<Self::TKey, Option<Self::TItem>>);
}

pub type Combo<TKey, TSkill> = Vec<(Vec<TKey>, TSkill)>;
