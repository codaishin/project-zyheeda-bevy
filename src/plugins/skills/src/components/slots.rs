use crate::{item::SkillItem, skills::Skill, slot_key::SlotKey};
use bevy::prelude::*;
use common::traits::accessors::get::GetRef;
use std::{collections::HashMap, fmt::Debug};

#[derive(Component, Clone, PartialEq, Debug)]
pub struct Slots(pub HashMap<SlotKey, Option<SkillItem>>);

impl Slots {
	pub fn new<const N: usize>(slots: [(SlotKey, Option<SkillItem>); N]) -> Self {
		Self(HashMap::from(slots))
	}
}

impl Default for Slots {
	fn default() -> Self {
		Self::new([])
	}
}

impl GetRef<SlotKey, SkillItem> for Slots {
	fn get(&self, key: &SlotKey) -> Option<&SkillItem> {
		let slot = self.0.get(key)?;
		slot.as_ref()
	}
}

impl GetRef<SlotKey, Handle<Skill>> for Slots {
	fn get(&self, key: &SlotKey) -> Option<&Handle<Skill>> {
		let item: &SkillItem = self.get(key)?;

		item.content.skill.as_ref()
	}
}

#[cfg(test)]
mod tests {
	use crate::item::SkillItemContent;

	use super::*;
	use bevy::utils::default;
	use common::{components::Side, test_tools::utils::new_handle};

	#[test]
	fn get_off_hand() {
		let slots = Slots(
			[(
				SlotKey::BottomHand(Side::Left),
				Some(SkillItem::named("my item")),
			)]
			.into(),
		);

		assert_eq!(
			Some(&SkillItem::named("my item")),
			slots.get(&SlotKey::BottomHand(Side::Left))
		);
	}

	#[test]
	fn get_main_hand() {
		let slots = Slots(
			[(
				SlotKey::BottomHand(Side::Right),
				Some(SkillItem::named("my item")),
			)]
			.into(),
		);

		assert_eq!(
			Some(&SkillItem::named("my item")),
			slots.get(&SlotKey::BottomHand(Side::Right))
		);
	}

	#[test]
	fn get_none() {
		let slots = Slots(
			[(
				SlotKey::BottomHand(Side::Right),
				Some(SkillItem::named("my item")),
			)]
			.into(),
		);

		assert_eq!(
			None::<&SkillItem>,
			slots.get(&SlotKey::BottomHand(Side::Left))
		);
	}

	#[test]
	fn get_skill_handle() {
		let handle = new_handle();
		let slots = Slots(
			[(
				SlotKey::BottomHand(Side::Right),
				Some(SkillItem::named("my item").with_content(SkillItemContent {
					skill: Some(handle.clone()),
					..default()
				})),
			)]
			.into(),
		);

		assert_eq!(Some(&handle), slots.get(&SlotKey::BottomHand(Side::Right)));
	}
}
