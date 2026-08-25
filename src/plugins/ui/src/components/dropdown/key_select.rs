use super::Dropdown;
use crate::{
	components::{
		combo_overview::ComboOverview,
		key_select::{AppendSkill, KeySelect},
	},
	tools::Layout,
	traits::{GetLayout, GetRootNode},
};
use bevy::prelude::*;

impl GetRootNode for Dropdown<KeySelect<AppendSkill>> {
	fn root_node(&self) -> Node {
		Node {
			position_type: PositionType::Absolute,
			top: Val::from(ComboOverview::MODIFY_BUTTON_DIMENSIONS.minimum_inner()),
			left: Val::from(ComboOverview::MODIFY_BUTTON_DIMENSIONS.width_inner()),
			..default()
		}
	}
}

impl<TExtra> GetLayout for Dropdown<KeySelect<TExtra>> {
	fn layout(&self) -> Layout {
		Layout::SINGLE_COLUMN
	}
}
