use crate::{tools::Layout, traits::UI};
use bevy::prelude::Component;

#[derive(Component, Default)]
pub(crate) struct Dropdown {
	pub(crate) layout: Layout,
	pub(crate) items: Vec<Box<dyn UI + Sync + Send>>,
}
