pub(crate) mod key_select;
pub(crate) mod skill_descriptor;

use bevy::prelude::{default, Component, Entity};
use std::{collections::HashSet, marker::PhantomData};

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
	pub(crate) child_dropdowns: HashSet<Entity>,
	pub(crate) source: Entity,
}

impl<TItem> DropdownUI<TItem> {
	pub(crate) fn new(source: Entity) -> Self {
		Self {
			source,
			child_dropdowns: default(),
			phantom_data: PhantomData,
		}
	}
}
