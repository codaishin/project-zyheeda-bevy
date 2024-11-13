use crate::{inventory_key::InventoryKey, item::SkillItem};

use common::{
	components::Collection,
	traits::accessors::get::{GetMut, GetRef},
};

pub type Inventory = Collection<Option<SkillItem>>;

impl GetRef<InventoryKey, SkillItem> for Inventory {
	fn get(&self, key: &InventoryKey) -> Option<&SkillItem> {
		let item = self.0.get(key.0)?;
		item.as_ref()
	}
}

impl GetMut<InventoryKey, Option<SkillItem>> for Inventory {
	fn get_mut(&mut self, InventoryKey(index): &InventoryKey) -> Option<&mut Option<SkillItem>> {
		let items = &mut self.0;

		if index >= &items.len() {
			fill(items, *index);
		}

		items.get_mut(*index)
	}
}

fn fill(inventory: &mut Vec<Option<SkillItem>>, inventory_key: usize) {
	let fill_len = inventory_key - inventory.len() + 1;
	for _ in 0..fill_len {
		inventory.push(None);
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_first_item() {
		let inventory = Inventory::new([Some(SkillItem::named("my item"))]);

		assert_eq!(
			Some(&SkillItem::named("my item")),
			inventory.get(&InventoryKey(0))
		);
	}

	#[test]
	fn get_none_when_empty() {
		let inventory = Inventory::new([]);

		assert_eq!(None, inventory.get(&InventoryKey(0)));
	}

	#[test]
	fn get_3rd_item() {
		let inventory = Inventory::new([None, None, Some(SkillItem::named("my item"))]);

		assert_eq!(
			Some(&SkillItem::named("my item")),
			inventory.get(&InventoryKey(2))
		);
	}

	#[test]
	fn get_item_mut() {
		let mut inventory = Inventory::new([Some(SkillItem::named("my item"))]);

		let item = inventory.get_mut(&InventoryKey(0));
		assert_eq!(Some(&mut Some(SkillItem::named("my item"))), item);
	}

	#[test]
	fn get_item_mut_exceeding_range() {
		let mut inventory = Inventory::new([Some(SkillItem::named("my item"))]);

		*inventory.get_mut(&InventoryKey(1)).expect("no item found") =
			Some(SkillItem::named("my other item"));

		assert_eq!(
			Inventory::new([
				Some(SkillItem::named("my item")),
				Some(SkillItem::named("my other item")),
			]),
			inventory
		);
	}
}
