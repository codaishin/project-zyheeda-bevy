use super::Accessor;
use crate::components::{Inventory, Item, SlotKey, Swap};

impl Accessor<Inventory, (SlotKey, Item), Item> for Swap {
	fn get_key_and_item(&self, inventory: &Inventory) -> Option<(SlotKey, Item)> {
		let inventory = &inventory.0;

		if self.inventory_key >= inventory.len() {
			return None;
		}
		inventory[self.inventory_key].map(|item| (self.slot_key, item))
	}

	fn with_item(&self, item: Option<Item>, inventory: &mut Inventory) -> Self {
		let inventory = &mut inventory.0;

		if self.inventory_key >= inventory.len() {
			fill_inventory(self, inventory);
		}
		inventory[self.inventory_key] = item;
		Swap {
			inventory_key: self.inventory_key,
			slot_key: self.slot_key,
		}
	}
}

fn fill_inventory(swap: &Swap, inventory: &mut Vec<Option<Item>>) {
	let fill_len = swap.inventory_key - inventory.len() + 1;
	inventory.extend(vec![None; fill_len]);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::Side;
	use bevy::prelude::default;

	#[test]
	fn get_item() {
		let slot_key = SlotKey::Legs;
		let item = Item {
			name: "my item",
			..default()
		};
		let inventory = Inventory::new([Some(item)]);
		let swap = Swap {
			inventory_key: 0,
			slot_key,
		};

		assert_eq!(Some((slot_key, item)), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn get_second_item() {
		let slot_key = SlotKey::Legs;
		let item = Item {
			name: "my second item",
			..default()
		};
		let inventory = Inventory::new([None, Some(item)]);
		let swap = Swap {
			inventory_key: 1,
			slot_key,
		};

		assert_eq!(Some((slot_key, item)), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn get_slot_key() {
		let slot_key = SlotKey::Hand(Side::Left);
		let item = Item {
			name: "my item",
			..default()
		};
		let inventory = Inventory::new([Some(item)]);
		let swap = Swap {
			inventory_key: 0,
			slot_key,
		};

		assert_eq!(Some((slot_key, item)), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn get_out_of_range() {
		let slot_key = SlotKey::Hand(Side::Left);
		let item = Item {
			name: "my item",
			..default()
		};
		let inventory = Inventory::new([Some(item)]);
		let swap = Swap {
			inventory_key: 42,
			slot_key,
		};

		assert_eq!(None, swap.get_key_and_item(&inventory));
	}

	#[test]
	fn copy_with_item() {
		let orig = Item {
			name: "orig",
			..default()
		};
		let replace = Item {
			name: "replace",
			..default()
		};
		let mut inventory = Inventory::new([Some(orig)]);
		let swap = Swap {
			inventory_key: 0,
			slot_key: SlotKey::Hand(Side::Left),
		};
		let copy = swap.with_item(Some(replace), &mut inventory);

		assert_eq!((swap, vec![Some(replace)]), (copy, inventory.0));
	}

	#[test]
	fn copy_with_item_push_back() {
		let item = Item {
			name: "my item",
			..default()
		};
		let new = Item {
			name: "new item",
			..default()
		};
		let mut inventory = Inventory::new([Some(item)]);
		let swap = Swap {
			inventory_key: 1,
			slot_key: SlotKey::Hand(Side::Left),
		};
		let copy = swap.with_item(Some(new), &mut inventory);

		assert_eq!((swap, vec![Some(item), Some(new)]), (copy, inventory.0));
	}
	#[test]
	fn copy_with_item_out_of_range() {
		let item = Item {
			name: "my item",
			..default()
		};
		let new = Item {
			name: "new item",
			..default()
		};
		let mut inventory = Inventory::new([Some(item)]);
		let swap = Swap {
			inventory_key: 3,
			slot_key: SlotKey::Hand(Side::Left),
		};
		let copy = swap.with_item(Some(new), &mut inventory);

		assert_eq!(
			(swap, vec![Some(item), None, None, Some(new)]),
			(copy, inventory.0)
		);
	}
}
