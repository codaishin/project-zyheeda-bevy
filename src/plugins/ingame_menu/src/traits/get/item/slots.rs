use crate::traits::get::Get;
use skills::components::{Item, SlotKey, Slots};

impl Get<SlotKey, Option<Item>> for Slots {
	fn get(&self, key: SlotKey) -> Option<Item> {
		self.0.get(&key).and_then(|slot| slot.item.clone())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{prelude::Entity, utils::default};
	use common::components::Side;
	use skills::components::Slot;

	#[test]
	fn get_off_hand() {
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Off),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						name: "my item",
						..default()
					}),
					combo_skill: None,
				},
			)]
			.into(),
		);

		assert_eq!(
			Some(Item {
				name: "my item",
				..default()
			}),
			slots.get(SlotKey::Hand(Side::Off))
		);
	}

	#[test]
	fn get_main_hand() {
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						name: "my item",
						..default()
					}),
					combo_skill: None,
				},
			)]
			.into(),
		);

		assert_eq!(
			Some(Item {
				name: "my item",
				..default()
			}),
			slots.get(SlotKey::Hand(Side::Main))
		);
	}

	#[test]
	fn get_none() {
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Main),
				Slot {
					entity: Entity::from_raw(42),
					item: Some(Item {
						name: "my item",
						..default()
					}),
					combo_skill: None,
				},
			)]
			.into(),
		);

		assert_eq!(None, slots.get(SlotKey::Hand(Side::Off)));
	}
}
