use super::{inspect_able::InspectAble, thread_safe::ThreadSafe};
use crate::tools::{
	change::Change,
	keys::slot::{Combo, SlotKey},
	skill_description::SkillToken,
	skill_icon::SkillIcon,
};
use bevy::prelude::*;
use std::collections::HashSet;

pub trait HandlesComboMenu {
	fn combos_with_skill<TSkill>() -> impl ConfigureCombos<TSkill>
	where
		for<'a> TSkill:
			InspectAble<SkillToken> + InspectAble<SkillIcon> + PartialEq + Clone + ThreadSafe;
}

pub trait ConfigureCombos<TSkill>
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
		TUpdateCombos: IntoSystem<In<Combo<Option<TSkill>>>, (), M2> + Copy,
		TCombos: GetCombosOrdered<TSkill> + GetComboAbleSkills<TSkill> + NextKeys + ThreadSafe;
}

pub trait GetComboAbleSkills<TSkill>
where
	TSkill: Clone,
{
	fn get_combo_able_skills(&self, key: &SlotKey) -> Vec<TSkill>;
}

pub trait NextKeys {
	fn next_keys(&self, combo_keys: &[SlotKey]) -> HashSet<SlotKey>;
}

pub trait GetCombosOrdered<TSkill> {
	fn combos_ordered(&self) -> Vec<Combo<TSkill>>;
}
