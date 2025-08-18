mod dto;

use crate::{components::slots::dto::SlotsDto, item::Item, traits::loadout_key::LoadoutKey};
use bevy::{asset::Handle, prelude::*};
use common::{
	tools::action_key::slot::SlotKey,
	traits::{accessors::get::GetRef, iterate::Iterate},
};
use macros::SavableComponent;
use std::{collections::HashMap, fmt::Debug};

#[derive(Component, SavableComponent, PartialEq, Debug, Clone)]
#[savable_component(dto = SlotsDto)]
pub struct Slots(pub HashMap<SlotKey, Option<Handle<Item>>>);

impl<T> From<T> for Slots
where
	T: IntoIterator<Item = (SlotKey, Option<Handle<Item>>)>,
{
	fn from(slots: T) -> Self {
		Self(HashMap::from_iter(slots))
	}
}

impl Default for Slots {
	fn default() -> Self {
		Self::from([])
	}
}

impl GetRef<SlotKey> for Slots {
	type TValue<'a>
		= &'a Handle<Item>
	where
		Self: 'a;

	fn get_ref(&self, key: &SlotKey) -> Option<&Handle<Item>> {
		let slot = self.0.get(key)?;
		slot.as_ref()
	}
}

impl LoadoutKey for Slots {
	type TKey = SlotKey;
}

impl<'a> Iterate<'a> for Slots {
	type TItem = (SlotKey, &'a Option<Handle<Item>>);
	type TIter = Iter<'a>;

	fn iterate(&'a self) -> Self::TIter {
		Iter { it: self.0.iter() }
	}
}

pub struct Iter<'a> {
	it: std::collections::hash_map::Iter<'a, SlotKey, Option<Handle<Item>>>,
}

impl<'a> Iterator for Iter<'a> {
	type Item = (SlotKey, &'a Option<Handle<Item>>);

	fn next(&mut self) -> Option<Self::Item> {
		let (key, slot) = self.it.next()?;
		Some((*key, slot))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use testing::new_handle;

	#[test]
	fn get_some() {
		let item = new_handle();
		let slots = Slots([(SlotKey(2), Some(item.clone()))].into());

		assert_eq!(Some(&item), slots.get_ref(&SlotKey(2)));
	}

	#[test]
	fn get_none() {
		let slots = Slots([(SlotKey(7), Some(new_handle()))].into());

		assert_eq!(None::<&Handle<Item>>, slots.get_ref(&SlotKey(11)));
	}
}
