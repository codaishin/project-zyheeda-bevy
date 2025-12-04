pub(crate) mod visualization;

mod dto;

use crate::{
	components::{
		active_slots::{ActiveSlots, Current, Old},
		slots::{dto::SlotsDto, visualization::SlotVisualization},
	},
	item::Item,
};
use bevy::{asset::Handle, prelude::*};
use common::{
	tools::action_key::slot::SlotKey,
	traits::{
		accessors::get::GetRef,
		iterate::Iterate,
		visible_slots::{EssenceSlot, ForearmSlot, HandSlot},
	},
};
use macros::SavableComponent;
use std::{collections::HashMap, fmt::Debug};

#[derive(Component, SavableComponent, PartialEq, Debug, Clone)]
#[require(
	SlotVisualization<HandSlot>,
	SlotVisualization<ForearmSlot>,
	SlotVisualization<EssenceSlot>,
	ActiveSlots<Current>,
	ActiveSlots<Old>,
)]
#[savable_component(dto = SlotsDto)]
pub struct Slots {
	pub(crate) items: HashMap<SlotKey, Option<Handle<Item>>>,
}

impl<T> From<T> for Slots
where
	T: IntoIterator<Item = (SlotKey, Option<Handle<Item>>)>,
{
	fn from(slots: T) -> Self {
		Self {
			items: HashMap::from_iter(slots),
		}
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
		let slot = self.items.get(key)?;
		slot.as_ref()
	}
}

impl<'a> Iterate<'a> for Slots {
	type TItem = (SlotKey, &'a Option<Handle<Item>>);
	type TIter = Iter<'a>;

	fn iterate(&'a self) -> Self::TIter {
		Iter {
			it: self.items.iter(),
		}
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

	mod get_handle {
		use super::*;

		#[test]
		fn get_some() {
			let item = new_handle();
			let slots = Slots::from([(SlotKey(2), Some(item.clone()))]);

			assert_eq!(Some(&item), slots.get_ref(&SlotKey(2)));
		}

		#[test]
		fn get_none() {
			let slots = Slots::from([(SlotKey(7), Some(new_handle()))]);

			assert_eq!(None::<&Handle<Item>>, slots.get_ref(&SlotKey(11)));
		}
	}
}
