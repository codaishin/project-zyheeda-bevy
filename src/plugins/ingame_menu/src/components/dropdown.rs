pub(crate) mod key_select;
pub(crate) mod skill_descriptor;

use bevy::prelude::{Component, Entity};
use std::marker::PhantomData;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct Dropdown<TItem> {
	pub(crate) items: Vec<TItem>,
}

impl<TItem> Default for Dropdown<TItem> {
	fn default() -> Self {
		Self {
			items: Default::default(),
		}
	}
}

#[derive(Component, Debug, PartialEq)]
pub(crate) struct DropdownUI<TItem> {
	phantom_data: PhantomData<TItem>,
	pub(crate) source: Entity,
}

impl<TItem> DropdownUI<TItem> {
	pub(crate) fn new(source: Entity) -> Self {
		Self {
			source,
			phantom_data: PhantomData,
		}
	}
}
