use crate::tools::Layout;
use bevy::prelude::Component;

#[derive(Component)]
pub(crate) struct Dropdown<TItem> {
	pub(crate) layout: Layout,
	pub(crate) items: Vec<TItem>,
}

impl<TItem> Default for Dropdown<TItem> {
	fn default() -> Self {
		Self {
			layout: Default::default(),
			items: Default::default(),
		}
	}
}
