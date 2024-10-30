use super::Dropdown;
use crate::{
	components::{
		combo_overview::ComboOverview,
		key_select::{AppendSkill, KeySelect, ReKeySkill},
	},
	tools::Layout,
	traits::{GetLayout, RootStyle},
};
use bevy::{
	prelude::default,
	ui::{PositionType, Style, Val},
};

impl RootStyle for Dropdown<KeySelect<ReKeySkill>> {
	fn root_style(&self) -> Style {
		Style {
			position_type: PositionType::Absolute,
			top: Val::from(ComboOverview::KEY_BUTTON_DIMENSIONS.height_inner()),
			left: Val::from(ComboOverview::KEY_BUTTON_DIMENSIONS.minimum_inner()),
			..default()
		}
	}
}

impl RootStyle for Dropdown<KeySelect<AppendSkill>> {
	fn root_style(&self) -> Style {
		Style {
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
