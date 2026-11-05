use super::{GetContainerNode, GetLayout, insert_ui_content::InsertUiContent};
use crate::{ApplyDropdown, Dropdown, resources::dropdown_change::DropdownChanged};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::prelude::*;

pub(crate) trait AddDropdown {
	fn add_dropdown<TLocalization, TItem>(&mut self) -> &mut Self
	where
		TLocalization: for<'w, 's> SystemParam<Item<'w, 's>: Localize> + ThreadSafe,
		TItem: InsertUiContent + Sync + Send + 'static,
		Dropdown<TItem>: GetContainerNode + GetLayout;
}

impl AddDropdown for App {
	fn add_dropdown<TLocalization, TItem>(&mut self) -> &mut Self
	where
		TLocalization: for<'w, 's> SystemParam<Item<'w, 's>: Localize> + ThreadSafe,
		TItem: InsertUiContent + Sync + Send + 'static,
		Dropdown<TItem>: GetContainerNode + GetLayout,
	{
		self.add_systems(
			Last,
			DropdownChanged::apply::<TLocalization, TItem>.in_set(ApplyDropdown),
		)
	}
}
