pub(crate) mod key_select;
pub(crate) mod skill_descriptor;

use crate::components::GlobalZIndexTop;
use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(IsDropdown)]
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

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct IsDropdown;

#[derive(Component, Debug, PartialEq)]
#[relationship_target(relationship = DropdownContainerOf)]
pub(crate) struct DropdownActive(Entity);

impl DropdownActive {
	pub(crate) fn container(&self) -> Entity {
		self.0
	}
}

#[derive(Component, Debug, PartialEq)]
#[relationship(relationship_target = DropdownActive)]
#[require(GlobalZIndexTop)]
pub(crate) struct DropdownContainerOf(pub(crate) Entity);
