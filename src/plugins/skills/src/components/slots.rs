use super::{Item, Slot, SlotKey};
use bevy::ecs::component::Component;
use common::traits::get::Get;
use std::collections::HashMap;

#[derive(Component, Clone, PartialEq, Debug)]
pub struct Slots(pub HashMap<SlotKey, Slot>);

impl Slots {
	pub fn new() -> Self {
		Self(HashMap::new())
	}
}

impl Default for Slots {
	fn default() -> Self {
		Self::new()
	}
}

impl Get<SlotKey, Option<Item>> for Slots {
	fn get(&self, key: &SlotKey) -> &Option<Item> {
		let Some(slot) = self.0.get(key) else {
			return &None;
		};
		&slot.item
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{prelude::Entity, utils::default};
	use common::components::Side;

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
				},
			)]
			.into(),
		);

		assert_eq!(
			&Some(Item {
				name: "my item",
				..default()
			}),
			slots.get(&SlotKey::Hand(Side::Off))
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
				},
			)]
			.into(),
		);

		assert_eq!(
			&Some(Item {
				name: "my item",
				..default()
			}),
			slots.get(&SlotKey::Hand(Side::Main))
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
				},
			)]
			.into(),
		);

		assert_eq!(&None, slots.get(&SlotKey::Hand(Side::Off)));
	}
}
