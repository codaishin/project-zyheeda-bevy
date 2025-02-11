use super::{
	accessors::get::{GetFieldRef, GetterRef},
	handles_equipment::CompatibleItems,
};
use crate::tools::{item_type::ItemType, slot_key::SlotKey};
use std::collections::HashMap;

pub trait IsCompatible<TSkill> {
	fn is_compatible(&self, key: &SlotKey, skill: &TSkill) -> bool;
}

pub struct CompatibilityChecker(pub HashMap<SlotKey, ItemType>);

impl<TSkill> IsCompatible<TSkill> for CompatibilityChecker
where
	TSkill: GetterRef<CompatibleItems>,
{
	fn is_compatible(&self, key: &SlotKey, skill: &TSkill) -> bool {
		let CompatibleItems(items) = CompatibleItems::get_field_ref(skill);

		let Some(key_item) = self.0.get(key) else {
			return false;
		};

		items.contains(key_item)
	}
}
