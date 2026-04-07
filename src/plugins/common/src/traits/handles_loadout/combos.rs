use crate::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::View,
		handles_loadout::skills::{GetSkillId, SkillIcon, SkillToken},
	},
};
use bevy::{ecs::component::Mutable, prelude::*};
use macros::EntityKey;
use std::{
	collections::HashSet,
	ops::{Deref, DerefMut},
};

pub type Combo<TKey, TSkill> = Vec<(Vec<TKey>, TSkill)>;

#[derive(EntityKey)]
pub struct Combos {
	pub entity: Entity,
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

pub trait ReadCombos<TId>:
	GetCombosOrdered<TSkill: GetSkillId<TId> + View<SkillToken> + View<SkillIcon>>
	+ NextConfiguredKeys<SlotKey>
{
}

impl<T, TId> ReadCombos<TId> for T where
	T: GetCombosOrdered<TSkill: GetSkillId<TId> + View<SkillToken> + View<SkillIcon>>
		+ NextConfiguredKeys<SlotKey>
{
}

pub trait UpdateCombos<TSkillID> {
	fn update_combos(&mut self, combos: Combo<SlotKey, Option<TSkillID>>);
}

impl<T, TSkillID> UpdateCombos<TSkillID> for T
where
	T: DerefMut<Target: Component<Mutability = Mutable> + UpdateCombos<TSkillID>>,
{
	fn update_combos(&mut self, combos: Combo<SlotKey, Option<TSkillID>>) {
		self.deref_mut().update_combos(combos)
	}
}
