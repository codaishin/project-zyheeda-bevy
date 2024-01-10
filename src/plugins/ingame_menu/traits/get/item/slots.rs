use crate::{
	components::{Item, SlotKey, Slots},
	plugins::ingame_menu::traits::get::Get,
};

impl Get<SlotKey, Option<Item>> for Slots {
	fn get(&self, key: SlotKey) -> Option<Item> {
		self.0.get(&key).and_then(|slot| slot.item.clone())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::{Side, Slot};
	use bevy::{prelude::Entity, utils::default};

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
	fn get_legs() {
		let slots = Slots(
			[(
				SlotKey::Legs,
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
			slots.get(SlotKey::Legs)
		);
	}

	#[test]
	fn get_none() {
		let slots = Slots(
			[(
				SlotKey::Legs,
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

		assert_eq!(None, slots.get(SlotKey::Hand(Side::Main)));
	}
}
