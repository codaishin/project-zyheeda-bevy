use super::{Item, Slot};
use crate::{items::slot_key::SlotKey, skills::Skill};
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

impl Get<SlotKey, Item> for Slots {
	fn get(&self, key: &SlotKey) -> Option<&Item> {
		let slot = self.0.get(key)?;
		slot.item.as_ref()
	}
}

impl Get<SlotKey, Skill> for Slots {
	fn get(&self, key: &SlotKey) -> Option<&Skill> {
		let item: &Item = self.get(key)?;
		item.skill.as_ref()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Mounts;
	use bevy::{prelude::Entity, utils::default};
	use common::components::Side;

	fn mounts() -> Mounts<Entity> {
		Mounts {
			hand: Entity::from_raw(100),
			forearm: Entity::from_raw(200),
		}
	}

	#[test]
	fn get_off_hand() {
		let slots = Slots(
			[(
				SlotKey::Hand(Side::Off),
				Slot {
					mounts: mounts(),
					item: Some(Item {
						name: "my item",
						..default()
					}),
				},
			)]
			.into(),
		);

		assert_eq!(
			Some(&Item {
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
					mounts: mounts(),
					item: Some(Item {
						name: "my item",
						..default()
					}),
				},
			)]
			.into(),
		);

		assert_eq!(
			Some(&Item {
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
					mounts: mounts(),
					item: Some(Item {
						name: "my item",
						..default()
					}),
				},
			)]
			.into(),
		);

		assert_eq!(None::<&Item>, slots.get(&SlotKey::Hand(Side::Off)));
	}
}
