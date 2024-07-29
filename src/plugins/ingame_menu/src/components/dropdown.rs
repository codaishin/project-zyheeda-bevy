pub(crate) mod key_select;
pub(crate) mod skill_descriptor;

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
