use crate::{
	items::{slot_key::SlotKey, Item},
	skills::Skill,
};
use bevy::asset::Handle;
use common::{components::Player, traits::accessor::Accessor};

impl Accessor<Player, (SlotKey, Option<Item<Handle<Skill>>>), Item<Handle<Skill>>>
	for (SlotKey, Option<Item<Handle<Skill>>>)
{
	fn get_key_and_item(&self, _: &Player) -> (SlotKey, Option<Item<Handle<Skill>>>) {
		self.clone()
	}

	fn with_item(&self, item: Option<Item<Handle<Skill>>>, _: &mut Player) -> Self {
		let (slot_key, ..) = *self;
		(slot_key, item)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;
	use common::components::Side;

	#[test]
	fn pull_slot_and_item() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "Some Item",
			..default()
		};

		let source = (slot_key, Some(item.clone()));

		assert_eq!((slot_key, Some(item)), source.get_key_and_item(&Player))
	}

	#[test]
	fn pull_none() {
		let slot_key = SlotKey::Hand(Side::Off);
		let source = (slot_key, None);

		assert_eq!((slot_key, None), source.get_key_and_item(&Player))
	}

	#[test]
	fn push_item() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "Some Item",
			..default()
		};
		let other_item = Item {
			name: "Other Item",
			..default()
		};

		let source = (slot_key, Some(item)).with_item(Some(other_item.clone()), &mut Player);

		assert_eq!((slot_key, Some(other_item)), source);
	}
}
