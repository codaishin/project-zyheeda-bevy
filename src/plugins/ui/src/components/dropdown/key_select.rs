use super::Dropdown;
use crate::{
	components::{
		combo_overview::ComboOverview,
		key_select::{AppendSkill, KeySelect},
	},
	tools::Layout,
	traits::{GetContainerNode, GetLayout},
};
use bevy::prelude::*;

impl GetContainerNode for Dropdown<KeySelect<AppendSkill>> {
	fn get_container_node(&self) -> Node {
		Node {
			position_type: PositionType::Absolute,
			top: Val::from(ComboOverview::MODIFY_BUTTON_DIMENSIONS.minimum_inner()),
			left: Val::from(ComboOverview::MODIFY_BUTTON_DIMENSIONS.width_inner()),
			..default()
		}
	}
}

impl<TExtra> GetLayout for Dropdown<KeySelect<TExtra>> {
	fn get_layout(&self) -> Layout {
		Layout::SINGLE_COLUMN
	}
}
