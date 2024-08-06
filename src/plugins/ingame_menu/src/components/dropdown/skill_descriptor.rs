use super::Dropdown;
use crate::{
	components::skill_descriptor::{DropdownItem, Horizontal, SkillDescriptor, Vertical},
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
			top: Val::Percent(0.),
			left: Val::Percent(100.),
			..default()
		}
	}
}

impl GetLayout for Dropdown<SkillDescriptor<DropdownItem<Vertical>>> {
	fn layout(&self) -> Layout {
		Layout::SINGLE_ROW
	}
}

impl RootStyle for Dropdown<SkillDescriptor<DropdownItem<Horizontal>>> {
	fn root_style(&self) -> Style {
		Style {
			position_type: PositionType::Absolute,
			top: Val::Percent(100.),
			right: Val::Percent(0.),
			..default()
		}
	}
}

impl GetLayout for Dropdown<SkillDescriptor<DropdownItem<Horizontal>>> {
	fn layout(&self) -> Layout {
		Layout::SINGLE_COLUMN
	}
}
