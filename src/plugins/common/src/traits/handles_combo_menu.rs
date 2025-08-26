use super::thread_safe::ThreadSafe;
use crate::{
	tools::{action_key::slot::PlayerSlot, change::Change},
	traits::{accessors::get::RefInto, handles_localization::Token},
};
use bevy::prelude::*;
use std::collections::HashSet;

pub type Combo<TKey, TSkill> = Vec<(Vec<TKey>, TSkill)>;

pub trait HandlesComboMenu {
	fn combos_with_skill<TSkill>() -> impl ConfigurePlayerCombos<TSkill>
	where
		TSkill: PartialEq
			+ Clone
			+ ThreadSafe
			+ for<'a> RefInto<'a, &'a Token>
			+ for<'a> RefInto<'a, &'a Option<Handle<Image>>>;
}

pub trait ConfigurePlayerCombos<TSkill>
where
	TSkill: PartialEq
		+ Clone
		+ ThreadSafe
		+ for<'a> RefInto<'a, &'a Token>
		+ for<'a> RefInto<'a, &'a Option<Handle<Image>>>,
{
	fn configure<TUpdateCombos, TCombos, M1, M2>(
		&self,
		app: &mut App,
		get_changed_combos: impl IntoSystem<(), Change<TCombos>, M1>,
		update_combos: TUpdateCombos,
	) where
		TUpdateCombos: IntoSystem<In<Combo<PlayerSlot, Option<TSkill>>>, (), M2> + Copy,
		TCombos: GetCombosOrdered<TSkill, PlayerSlot>
			+ GetComboAblePlayerSkills<TSkill>
			+ NextConfiguredKeys<PlayerSlot>
			+ ThreadSafe;
}

pub trait GetComboAblePlayerSkills<TSkill>
where
	TSkill: Clone,
{
	fn get_combo_able_player_skills(&self, key: &PlayerSlot) -> Vec<TSkill>;
}

pub trait NextConfiguredKeys<TKey> {
	fn next_keys(&self, combo_keys: &[TKey]) -> HashSet<TKey>;
}

pub trait GetCombosOrdered<TSkill, TKey> {
	/// Get combos with a consistent ordering.
	/// The specific ordering heuristic is up to the implementor.
	fn combos_ordered(&self) -> Vec<Vec<(Vec<TKey>, TSkill)>>;
}
