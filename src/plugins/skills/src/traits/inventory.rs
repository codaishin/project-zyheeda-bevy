use crate::{
	components::inventory::Inventory,
	items::{InventoryKey, Item, SlotKey},
};
use common::{components::Swap, traits::accessor::Accessor};

impl Accessor<Inventory, (SlotKey, Option<Item>), Item> for Swap<InventoryKey, SlotKey> {
	fn get_key_and_item(&self, inventory: &Inventory) -> (SlotKey, Option<Item>) {
		let inventory = &inventory.0;
		let Swap(inventory_key, slot_key) = *self;

		if inventory_key.0 >= inventory.len() {
			return (slot_key, None);
		}
		(slot_key, inventory[inventory_key.0].clone())
	}

	fn with_item(&self, item: Option<Item>, inventory: &mut Inventory) -> Self {
		let inventory = &mut inventory.0;
		let Swap(inventory_key, slot_key) = self;

		if inventory_key.0 >= inventory.len() {
			fill_inventory(inventory_key, inventory);
		}
		inventory[inventory_key.0] = item;
		Self(*inventory_key, *slot_key)
	}
}

impl Accessor<Inventory, (SlotKey, Option<Item>), Item> for Swap<SlotKey, InventoryKey> {
	fn get_key_and_item(&self, inventory: &Inventory) -> (SlotKey, Option<Item>) {
		Swap(self.1, self.0).get_key_and_item(inventory)
	}

	fn with_item(&self, item: Option<Item>, inventory: &mut Inventory) -> Self {
		let Swap(inventory_key, slot_key) = Swap(self.1, self.0).with_item(item, inventory);
		Self(slot_key, inventory_key)
	}
}

fn fill_inventory(inventory_key: &InventoryKey, inventory: &mut Vec<Option<Item>>) {
	let fill_len = inventory_key.0 - inventory.len() + 1;
	inventory.extend(vec![None; fill_len]);
}

#[cfg(test)]
mod test_swap_inventory_key_slot_key {
	use super::*;
	use bevy::prelude::default;
	use common::components::Side;

	#[test]
	fn get_item() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "my item",
			..default()
		};
		let inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(InventoryKey(0), slot_key);

		assert_eq!((slot_key, Some(item)), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn get_second_item() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "my second item",
			..default()
		};
		let inventory = Inventory::new([None, Some(item.clone())]);
		let swap = Swap(InventoryKey(1), slot_key);

		assert_eq!((slot_key, Some(item)), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn get_slot_key() {
		let slot_key = SlotKey::Hand(Side::Main);
		let item = Item {
			name: "my item",
			..default()
		};
		let inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(InventoryKey(0), slot_key);

		assert_eq!((slot_key, Some(item)), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn get_out_of_range() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "my item",
			..default()
		};
		let inventory = Inventory::new([Some(item)]);
		let swap = Swap(InventoryKey(42), slot_key);

		assert_eq!((slot_key, None), swap.get_key_and_item(&inventory));
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
		let swap = Swap(InventoryKey(0), SlotKey::Hand(Side::Off));
		let copy = swap.with_item(Some(replace.clone()), &mut inventory);

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
		let mut inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(InventoryKey(1), SlotKey::Hand(Side::Off));
		let copy = swap.with_item(Some(new.clone()), &mut inventory);

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
		let mut inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(InventoryKey(3), SlotKey::Hand(Side::Off));
		let copy = swap.with_item(Some(new.clone()), &mut inventory);

		assert_eq!(
			(swap, vec![Some(item), None, None, Some(new)]),
			(copy, inventory.0)
		);
	}
}

#[cfg(test)]
mod test_swap_slot_key_inventory_key {
	use super::*;
	use bevy::prelude::default;
	use common::components::Side;

	#[test]
	fn get_item() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "my item",
			..default()
		};
		let inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(slot_key, InventoryKey(0));

		assert_eq!((slot_key, Some(item)), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn get_second_item() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "my second item",
			..default()
		};
		let inventory = Inventory::new([None, Some(item.clone())]);
		let swap = Swap(slot_key, InventoryKey(1));

		assert_eq!((slot_key, Some(item)), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn get_slot_key() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "my item",
			..default()
		};
		let inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(slot_key, InventoryKey(0));

		assert_eq!((slot_key, Some(item)), swap.get_key_and_item(&inventory));
	}

	#[test]
	fn get_out_of_range() {
		let slot_key = SlotKey::Hand(Side::Off);
		let item = Item {
			name: "my item",
			..default()
		};
		let inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(slot_key, InventoryKey(42));

		assert_eq!((slot_key, None), swap.get_key_and_item(&inventory));
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
		let swap = Swap(SlotKey::Hand(Side::Off), InventoryKey(0));
		let copy = swap.with_item(Some(replace.clone()), &mut inventory);

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
		let mut inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(SlotKey::Hand(Side::Off), InventoryKey(1));
		let copy = swap.with_item(Some(new.clone()), &mut inventory);

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
		let mut inventory = Inventory::new([Some(item.clone())]);
		let swap = Swap(SlotKey::Hand(Side::Off), InventoryKey(3));
		let copy = swap.with_item(Some(new.clone()), &mut inventory);

		assert_eq!(
			(swap, vec![Some(item), None, None, Some(new)]),
			(copy, inventory.0)
		);
	}
}
