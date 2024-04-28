use super::{InventoryKey, Item};
use common::{components::Collection, traits::get::Get};

pub type Inventory = Collection<Option<Item>>;

impl Get<InventoryKey, Option<Item>> for Inventory {
	fn get(&self, key: &InventoryKey) -> &Option<Item> {
		let Some(item) = self.0.get(key.0) else {
			return &None;
		};
		item
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::utils::default;

	#[test]
	fn get_first_item() {
		let inventory = Inventory::new([Some(Item {
			name: "my item",
			..default()
		})]);

		assert_eq!(
			&Some(Item {
				name: "my item",
				..default()
			}),
			inventory.get(&InventoryKey(0))
		);
	}

	#[test]
	fn get_none_when_empty() {
		let inventory = Inventory::new([]);

		assert_eq!(&None, inventory.get(&InventoryKey(0)));
	}

	#[test]
	fn get_3rd_item() {
		let inventory = Inventory::new([
			None,
			None,
			Some(Item {
				name: "my item",
				..default()
			}),
		]);

		assert_eq!(
			&Some(Item {
				name: "my item",
				..default()
			}),
			inventory.get(&InventoryKey(2))
		);
	}
}
