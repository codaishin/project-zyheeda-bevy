use super::Dropdown;
use crate::{
	components::{
		combo_overview::ComboOverview,
		skill_descriptor::{DropdownItem, Horizontal, SkillDescriptor, Vertical},
	},
	tools::Layout,
	traits::{GetLayout, RootStyle},
};
use bevy::{
	prelude::default,
	ui::{PositionType, Style, Val},
};

impl RootStyle for Dropdown<SkillDescriptor<DropdownItem<Vertical>>> {
	fn root_style(&self) -> Style {
		Style {
			position_type: PositionType::Absolute,
			top: Val::from(ComboOverview::SKILL_BUTTON_DIMENSIONS.nested_height()),
			left: Val::from(ComboOverview::SKILL_BUTTON_DIMENSIONS.nested_minimum()),
			..default()
		}
	}
}

impl GetLayout for Dropdown<SkillDescriptor<DropdownItem<Vertical>>> {
	fn layout(&self) -> Layout {
		Layout::SINGLE_COLUMN
	}
}

impl RootStyle for Dropdown<SkillDescriptor<DropdownItem<Horizontal>>> {
	fn root_style(&self) -> Style {
		Style {
			position_type: PositionType::Absolute,
			top: Val::from(ComboOverview::KEY_BUTTON_DIMENSIONS.nested_minimum()),
			left: Val::from(ComboOverview::KEY_BUTTON_DIMENSIONS.nested_width()),
			..default()
		}
	}
}

impl GetLayout for Dropdown<SkillDescriptor<DropdownItem<Horizontal>>> {
	fn layout(&self) -> Layout {
		Layout::SINGLE_ROW
	}
}
