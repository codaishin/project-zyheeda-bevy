mod dto;

use crate::{components::inventory::dto::InventoryDto, item::Item};
use bevy::prelude::*;
use common::{tools::inventory_key::InventoryKey, traits::iterate::Iterate};
use macros::SavableComponent;
use std::iter::Enumerate;

#[derive(Component, SavableComponent, Debug, PartialEq, Default, Clone)]
#[savable_component(dto = InventoryDto)]
pub struct Inventory(pub(crate) Vec<Option<Handle<Item>>>);

impl Inventory {
	pub(crate) fn fill_up_to(&mut self, index: usize) {
		if index < self.0.len() {
			return;
		}
		self.0.resize_with(index + 1, || None);
	}
}

impl<T> From<T> for Inventory
where
	T: IntoIterator<Item = Option<Handle<Item>>>,
{
	fn from(items: T) -> Self {
		Self(Vec::from_iter(items))
	}
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
