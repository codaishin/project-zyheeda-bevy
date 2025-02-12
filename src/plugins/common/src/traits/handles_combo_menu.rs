use super::{
	accessors::get::{GetFieldRef, GetterRef},
	handles_equipment::{Combo, CompatibleItems},
};
use crate::tools::{item_type::ItemType, slot_key::SlotKey};
use std::collections::{HashMap, HashSet};

pub trait IsCompatible<TSkill> {
	fn is_compatible(&self, key: &SlotKey, skill: &TSkill) -> bool;
}

pub trait NextKeys {
	fn next_keys(&self, combo_keys: &[SlotKey]) -> HashSet<SlotKey>;
}

pub trait GetCombosOrdered<TSkill> {
	fn combos_ordered(&self) -> Vec<Combo<TSkill>>;
}

// This should go later into the skills plugin
pub struct EquipmentDescriptor<TSkill> {
	pub item_types: HashMap<SlotKey, ItemType>,
	pub combo_keys: HashSet<Vec<SlotKey>>,
	pub combos: Vec<Combo<TSkill>>,
}

impl<TSkill> IsCompatible<TSkill> for EquipmentDescriptor<TSkill>
where
	TSkill: GetterRef<CompatibleItems>,
{
	fn is_compatible(&self, key: &SlotKey, skill: &TSkill) -> bool {
		let CompatibleItems(compatible_items) = CompatibleItems::get_field_ref(skill);

		let Some(item_type) = self.item_types.get(key) else {
			return false;
		};

		compatible_items.contains(item_type)
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
	fn combos_ordered(&self) -> Vec<Combo<TSkill>> {
		self.combos.clone()
	}
}
