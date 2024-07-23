use crate::tools::Layout;
use bevy::{prelude::Component, ui::Style};

#[derive(Component)]
pub(crate) struct Dropdown<TItem> {
	pub(crate) layout: Layout,
	pub(crate) style: Style,
	pub(crate) items: Vec<TItem>,
}

impl<TItem> Default for Dropdown<TItem> {
	fn default() -> Self {
		Self {
			layout: Default::default(),
			style: Default::default(),
			items: Default::default(),
		}
	}
}
