use crate::{inventory_key::InventoryKey, item::SkillItem};
use bevy::asset::Handle;
use common::{
	components::Collection,
	traits::accessors::get::{GetMut, GetRef},
};

pub type Inventory = Collection<Option<Handle<SkillItem>>>;

impl GetRef<InventoryKey, Handle<SkillItem>> for Inventory {
	fn get(&self, key: &InventoryKey) -> Option<&Handle<SkillItem>> {
		let item = self.0.get(key.0)?;
		item.as_ref()
	}
}

impl GetMut<InventoryKey, Option<Handle<SkillItem>>> for Inventory {
	fn get_mut(
		&mut self,
		InventoryKey(index): &InventoryKey,
	) -> Option<&mut Option<Handle<SkillItem>>> {
		let items = &mut self.0;

		if index >= &items.len() {
			fill(items, *index);
		}

		items.get_mut(*index)
	}
}

fn fill(inventory: &mut Vec<Option<Handle<SkillItem>>>, inventory_key: usize) {
	let fill_len = inventory_key - inventory.len() + 1;
	for _ in 0..fill_len {
		inventory.push(None);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::new_handle;

	#[test]
	fn get_first_item() {
		let item = new_handle();
		let inventory = Inventory::new([Some(item.clone())]);

		assert_eq!(Some(&item), inventory.get(&InventoryKey(0)));
	}

	#[test]
	fn get_none_when_empty() {
		let inventory = Inventory::new([]);

		assert_eq!(None, inventory.get(&InventoryKey(0)));
	}

	#[test]
	fn get_3rd_item() {
		let item = new_handle();
		let inventory = Inventory::new([None, None, Some(item.clone())]);

		assert_eq!(Some(&item), inventory.get(&InventoryKey(2)));
	}

	#[test]
	fn get_item_mut() {
		let item = new_handle();
		let mut inventory = Inventory::new([Some(item.clone())]);

		assert_eq!(Some(&mut Some(item)), inventory.get_mut(&InventoryKey(0)));
	}

	#[test]
	fn get_item_mut_exceeding_range() {
		let item = new_handle();
		let mut inventory = Inventory::new([Some(item.clone())]);

		let new_item = new_handle();
		*inventory.get_mut(&InventoryKey(1)).expect("no item found") = Some(new_item.clone());

		assert_eq!(Inventory::new([Some(item), Some(new_item),]), inventory);
	}

	#[test]
	fn get_item_mut_exceeding_range_with_gaps() {
		let item = new_handle();
		let mut inventory = Inventory::new([Some(item.clone())]);

		let new_item = new_handle();
		*inventory.get_mut(&InventoryKey(2)).expect("no item found") = Some(new_item.clone());

		assert_eq!(
			Inventory::new([Some(item), None, Some(new_item),]),
			inventory
		);
	}
}
