pub(crate) mod skill_select;

use bevy::prelude::Component;

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
