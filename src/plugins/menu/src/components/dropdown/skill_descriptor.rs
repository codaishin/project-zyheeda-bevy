use super::Dropdown;
use crate::{
	components::{
		combo_overview::ComboOverview,
		skill_button::{DropdownItem, Horizontal, SkillButton, Vertical},
	},
	tools::Layout,
	traits::{GetLayout, GetRootNode},
};
use bevy::prelude::*;

impl GetRootNode for Dropdown<SkillButton<DropdownItem<Vertical>>> {
	fn root_node(&self) -> Node {
		Node {
			position_type: PositionType::Absolute,
			top: Val::from(ComboOverview::SKILL_BUTTON_DIMENSIONS.height_inner()),
			left: Val::from(ComboOverview::SKILL_BUTTON_DIMENSIONS.minimum_inner()),
			..default()
		}
	}
}

impl GetLayout for Dropdown<SkillButton<DropdownItem<Vertical>>> {
	fn layout(&self) -> Layout {
		Layout::SINGLE_COLUMN
	}
}

impl GetRootNode for Dropdown<SkillButton<DropdownItem<Horizontal>>> {
	fn root_node(&self) -> Node {
		Node {
			position_type: PositionType::Absolute,
			top: Val::from(ComboOverview::KEY_BUTTON_DIMENSIONS.minimum_inner()),
			left: Val::from(ComboOverview::KEY_BUTTON_DIMENSIONS.width_inner()),
			..default()
		}
	}
}

impl GetLayout for Dropdown<SkillButton<DropdownItem<Horizontal>>> {
	fn layout(&self) -> Layout {
		Layout::SINGLE_ROW
	}
}
