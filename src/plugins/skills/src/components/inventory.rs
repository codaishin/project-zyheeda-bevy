mod dto;

use crate::{
	components::inventory::dto::InventoryDto,
	item::Item,
	traits::loadout_key::LoadoutKey,
};
use bevy::{asset::Handle, ecs::component::Component};
use common::{
	tools::inventory_key::InventoryKey,
	traits::{
		accessors::get::{GetMut, GetRef},
		iterate::Iterate,
	},
};
use macros::SavableComponent;
use std::iter::Enumerate;

#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone)]
#[savable_component(dto = InventoryDto)]
pub struct Inventory(pub(crate) Vec<Option<Handle<Item>>>);

impl<T> From<T> for Inventory
where
	T: IntoIterator<Item = Option<Handle<Item>>>,
{
	fn from(items: T) -> Self {
		Self(Vec::from_iter(items))
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

impl<'a> Iterate<'a> for Inventory {
	type TItem = (InventoryKey, &'a Option<Handle<Item>>);
	type TIter = Iter<'a>;

	fn iterate(&'a self) -> Self::TIter {
		Iter {
			it: self.0.iter().enumerate(),
		}
	}
}

pub struct Iter<'a> {
	it: Enumerate<std::slice::Iter<'a, Option<Handle<Item>>>>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = (InventoryKey, &'a Option<Handle<Item>>);

	fn next(&mut self) -> Option<Self::Item> {
		let (i, item) = self.it.next()?;
		Some((InventoryKey(i), item))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::new_handle;

	#[test]
	fn get_first_item() {
		let item = new_handle();
		let inventory = Inventory::from([Some(item.clone())]);

		assert_eq!(Some(&item), inventory.get(&InventoryKey(0)));
	}

	#[test]
	fn get_none_when_empty() {
		let inventory = Inventory::from([]);

		assert_eq!(None, inventory.get(&InventoryKey(0)));
	}

	#[test]
	fn get_3rd_item() {
		let item = new_handle();
		let inventory = Inventory::from([None, None, Some(item.clone())]);

		assert_eq!(Some(&item), inventory.get(&InventoryKey(2)));
	}

	#[test]
	fn get_item_mut() {
		let item = new_handle();
		let mut inventory = Inventory::from([Some(item.clone())]);

		assert_eq!(Some(&mut Some(item)), inventory.get_mut(&InventoryKey(0)));
	}

	#[test]
	fn get_item_mut_exceeding_range() {
		let item = new_handle();
		let mut inventory = Inventory::from([Some(item.clone())]);

		let new_item = new_handle();
		*inventory.get_mut(&InventoryKey(1)).expect("no item found") = Some(new_item.clone());

		assert_eq!(Inventory::from([Some(item), Some(new_item),]), inventory);
	}

	#[test]
	fn get_item_mut_exceeding_range_with_gaps() {
		let item = new_handle();
		let mut inventory = Inventory::from([Some(item.clone())]);

		let new_item = new_handle();
		*inventory.get_mut(&InventoryKey(2)).expect("no item found") = Some(new_item.clone());

		assert_eq!(
			Inventory::from([Some(item), None, Some(new_item),]),
			inventory
		);
	}
}
