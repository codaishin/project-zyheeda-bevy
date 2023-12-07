use super::Accessor;
use crate::components::{Item, Player, SlotKey};

impl Accessor<Player, (SlotKey, Item), Item> for (SlotKey, Option<Item>) {
	fn get_key_and_item(&self, _: &Player) -> Option<(SlotKey, Item)> {
		let (slot_key, item) = *self;
		let item = item?;
		Some((slot_key, item))
	}

	fn with_item(&self, item: Option<Item>, _: &mut Player) -> Self {
		let (slot_key, ..) = *self;
		(slot_key, item)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Side;
	use bevy::utils::default;

	#[test]
	fn pull_slot_and_item() {
		let slot_key = SlotKey::Hand(Side::Left);
		let item = Item {
			name: "Some Item",
			..default()
		};

		let source = (slot_key, Some(item));

		assert_eq!(
			Some((slot_key, item)),
			source.get_key_and_item(&Player::default())
		)
	}

	#[test]
	fn pull_none() {
		let slot_key = SlotKey::Hand(Side::Left);
		let source = (slot_key, None);

		assert_eq!(None, source.get_key_and_item(&Player::default()))
	}

	#[test]
	fn push_item() {
		let slot_key = SlotKey::Hand(Side::Left);
		let item = Item {
			name: "Some Item",
			..default()
		};
		let other_item = Item {
			name: "Other Item",
			..default()
		};

		let source = (slot_key, Some(item)).with_item(Some(other_item), &mut Player::default());

		assert_eq!((slot_key, Some(other_item)), source);
	}
}
