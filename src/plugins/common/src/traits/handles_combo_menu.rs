use super::{inspect_able::InspectAble, thread_safe::ThreadSafe};
use crate::tools::{
	skill_description::SkillDescription,
	skill_icon::SkillIcon,
	slot_key::{Combo, SlotKey},
};
use bevy::prelude::*;
use std::collections::HashSet;

pub trait HandlesComboMenu {
	fn combos_with_skill<TSkill>() -> impl ConfigureCombos<TSkill>
	where
		for<'a> TSkill:
			InspectAble<SkillDescription> + InspectAble<SkillIcon> + PartialEq + Clone + ThreadSafe;
}

pub trait ConfigureCombos<TSkill>
where
	for<'a> TSkill:
		InspectAble<SkillDescription> + InspectAble<SkillIcon> + PartialEq + Clone + ThreadSafe,
{
	fn configure<TUpdateCombos, TEquipment, M1, M2>(
		&self,
		app: &mut App,
		get_equipment_info: impl IntoSystem<(), Option<TEquipment>, M1>,
		update_combos: TUpdateCombos,
	) where
		TUpdateCombos: IntoSystem<In<Combo<Option<TSkill>>>, (), M2> + Copy,
		TEquipment: GetCombosOrdered<TSkill> + GetComboAbleSkills<TSkill> + NextKeys + ThreadSafe;
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
