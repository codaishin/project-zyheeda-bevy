use super::{inspect_able::InspectAble, thread_safe::ThreadSafe};
use crate::tools::{
	action_key::slot::PlayerSlot,
	change::Change,
	skill_description::SkillToken,
	skill_icon::SkillIcon,
};
use bevy::prelude::*;
use std::collections::HashSet;

pub type Combo<TKey, TSkill> = Vec<(Vec<TKey>, TSkill)>;

pub trait HandlesComboMenu {
	fn combos_with_skill<TSkill>() -> impl ConfigurePlayerCombos<TSkill>
	where
		for<'a> TSkill:
			InspectAble<SkillToken> + InspectAble<SkillIcon> + PartialEq + Clone + ThreadSafe;
}

pub trait ConfigurePlayerCombos<TSkill>
where
	for<'a> TSkill:
		InspectAble<SkillToken> + InspectAble<SkillIcon> + PartialEq + Clone + ThreadSafe,
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
			+ NextKeys<PlayerSlot>
			+ ThreadSafe;
}

pub trait GetComboAblePlayerSkills<TSkill>
where
	TSkill: Clone,
{
	fn get_combo_able_player_skills(&self, key: &PlayerSlot) -> Vec<TSkill>;
}

pub trait NextKeys<TKey> {
	fn next_keys(&self, combo_keys: &[TKey]) -> HashSet<TKey>;
}

pub trait GetCombosOrdered<TSkill, TKey> {
	fn combos_ordered(&self) -> Vec<Vec<(Vec<TKey>, TSkill)>>;
}
