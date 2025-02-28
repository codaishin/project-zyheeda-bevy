use crate::{item::Item, traits::loadout_key::LoadoutKey};
use bevy::{asset::Handle, ecs::component::Component};
use common::{
	tools::inventory_key::InventoryKey,
	traits::{
		accessors::get::{GetMut, GetRef},
		iterate::Iterate,
	},
};

#[derive(Component, Debug, PartialEq, Default)]
pub struct Inventory(pub(crate) Vec<Option<Handle<Item>>>);

impl Inventory {
	pub fn new<const N: usize>(items: [Option<Handle<Item>>; N]) -> Self {
		Self(Vec::from(items))
	}
}

impl GetRef<InventoryKey, Handle<Item>> for Inventory {
	fn get(&self, key: &InventoryKey) -> Option<&Handle<Item>> {
		let item = self.0.get(key.0)?;
		item.as_ref()
	}
}

impl GetMut<InventoryKey, Option<Handle<Item>>> for Inventory {
	fn get_mut(&mut self, InventoryKey(index): &InventoryKey) -> Option<&mut Option<Handle<Item>>> {
		let items = &mut self.0;

		if index >= &items.len() {
			fill(items, *index);
		}

		items.get_mut(*index)
	}
}

fn fill(inventory: &mut Vec<Option<Handle<Item>>>, inventory_key: usize) {
	let fill_len = inventory_key - inventory.len() + 1;
	for _ in 0..fill_len {
		inventory.push(None);
	}
}

impl LoadoutKey for Inventory {
	type TKey = InventoryKey;
}

impl Iterate for Inventory {
	type TItem<'a>
		= (InventoryKey, &'a Option<Handle<Item>>)
	where
		Self: 'a;

	fn iterate(&self) -> impl Iterator<Item = Self::TItem<'_>> {
		self.0
			.iter()
			.enumerate()
			.map(|(i, item)| (InventoryKey(i), item))
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
