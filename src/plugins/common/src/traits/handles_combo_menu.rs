use super::{handles_equipment::Combo, thread_safe::ThreadSafe};
use crate::tools::slot_key::SlotKey;
use bevy::prelude::*;
use std::collections::{HashMap, HashSet};

pub trait HandlesComboMenu {
	fn combos_with_skill<TSkill>() -> impl ConfigureCombos<TSkill>
	where
		TSkill: PartialEq + Clone + ThreadSafe;
}

pub trait ConfigureCombos<TSkill>
where
	TSkill: PartialEq + Clone + ThreadSafe,
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
	fn get_combo_able_skills(&self, key: &SlotKey) -> Vec<ComboSkillDescriptor<TSkill>>;
}

pub trait NextKeys {
	fn next_keys(&self, combo_keys: &[SlotKey]) -> HashSet<SlotKey>;
}

pub trait GetCombosOrdered<TSkill> {
	fn combos_ordered(&self) -> Vec<Combo<ComboSkillDescriptor<TSkill>>>;
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct ComboSkillDescriptor<TSkill> {
	pub skill: TSkill,
	pub icon: Option<Handle<Image>>,
	pub name: String,
}

// This should go later into the skills plugin
#[derive(Debug, PartialEq)]
pub struct EquipmentDescriptor<TSkill> {
	pub compatible_skills: HashMap<SlotKey, Vec<ComboSkillDescriptor<TSkill>>>,
	pub combo_keys: HashSet<Vec<SlotKey>>,
	pub combos: Vec<Combo<ComboSkillDescriptor<TSkill>>>,
}

impl<TSkill> GetComboAbleSkills<TSkill> for EquipmentDescriptor<TSkill>
where
	TSkill: Clone,
{
	fn get_combo_able_skills(&self, key: &SlotKey) -> Vec<ComboSkillDescriptor<TSkill>> {
		self.compatible_skills.get(key).cloned().unwrap_or_default()
	}
}

impl<TSkill> NextKeys for EquipmentDescriptor<TSkill> {
	fn next_keys(&self, combo_keys: &[SlotKey]) -> HashSet<SlotKey> {
		self.combo_keys
			.iter()
			.filter(|combo| combo.starts_with(combo_keys))
			.filter_map(|combo| combo.get(combo_keys.len()))
			.cloned()
			.collect()
	}
}

impl<TSkill> GetCombosOrdered<TSkill> for EquipmentDescriptor<TSkill>
where
	TSkill: Clone,
{
	fn combos_ordered(&self) -> Vec<Combo<ComboSkillDescriptor<TSkill>>> {
		self.combos.clone()
	}
}
