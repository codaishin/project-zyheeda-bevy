use crate::{item::SkillItem, slot_key::SlotKey};
use bevy::prelude::*;
use common::traits::accessors::get::GetRef;
use std::{collections::HashMap, fmt::Debug};

#[derive(Component, Clone, PartialEq, Debug)]
pub struct Slots(pub HashMap<SlotKey, Option<Handle<SkillItem>>>);

impl Slots {
	pub fn new<const N: usize>(slots: [(SlotKey, Option<Handle<SkillItem>>); N]) -> Self {
		Self(HashMap::from(slots))
	}
}

impl Default for Slots {
	fn default() -> Self {
		Self::new([])
	}
}

impl GetRef<SlotKey, Handle<SkillItem>> for Slots {
	fn get(&self, key: &SlotKey) -> Option<&Handle<SkillItem>> {
		let slot = self.0.get(key)?;
		slot.as_ref()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::{components::Side, test_tools::utils::new_handle};

	#[test]
	fn get_off_hand() {
		let item = new_handle();
		let slots = Slots([(SlotKey::BottomHand(Side::Left), Some(item.clone()))].into());

		assert_eq!(Some(&item), slots.get(&SlotKey::BottomHand(Side::Left)));
	}

	#[test]
	fn get_main_hand() {
		let item = new_handle();
		let slots = Slots([(SlotKey::BottomHand(Side::Right), Some(item.clone()))].into());

		assert_eq!(Some(&item), slots.get(&SlotKey::BottomHand(Side::Right)));
	}

	#[test]
	fn get_none() {
		let slots = Slots([(SlotKey::BottomHand(Side::Right), Some(new_handle()))].into());

		assert_eq!(
			None::<&Handle<SkillItem>>,
			slots.get(&SlotKey::BottomHand(Side::Left))
		);
	}
}
