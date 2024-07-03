use crate::{tools::Layout, traits::UI};
use bevy::{prelude::Component, ui::Style};

#[derive(Component, Default)]
pub(crate) struct Dropdown {
	pub(crate) layout: Layout,
	pub(crate) style: Style,
	pub(crate) items: Vec<Box<dyn UI + Sync + Send>>,
}
