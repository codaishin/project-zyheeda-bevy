use super::{
	accessors::get::{GetFieldRef, GetterRef},
	handles_equipment::CompatibleItems,
};
use crate::tools::{item_type::ItemType, slot_key::SlotKey};
use std::collections::{HashMap, HashSet};

pub trait IsCompatible<TSkill> {
	fn is_compatible(&self, key: &SlotKey, skill: &TSkill) -> bool;
}

pub trait NextKeys {
	fn next_keys(&self, combo_keys: &[SlotKey]) -> HashSet<SlotKey>;
}

// This should go later into the skills plugin
pub struct EquipmentDescriptor {
	pub item_types: HashMap<SlotKey, ItemType>,
	pub combos: HashSet<Vec<SlotKey>>,
}

impl<TSkill> IsCompatible<TSkill> for EquipmentDescriptor
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

impl NextKeys for EquipmentDescriptor {
	fn next_keys(&self, combo_keys: &[SlotKey]) -> HashSet<SlotKey> {
		let mut next_keys = HashSet::default();

		for combo in self.combos.iter() {
			if !combo.starts_with(combo_keys) {
				continue;
			}
			let Some(next) = combo.get(combo_keys.len()) else {
				continue;
			};
			next_keys.insert(*next);
		}

		next_keys
	}
}
