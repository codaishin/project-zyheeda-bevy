use super::Dropdown;
use crate::{
	components::key_select::KeySelect,
	tools::Layout,
	traits::{GetLayout, RootStyle},
};
use bevy::{
	prelude::default,
	ui::{PositionType, Style, Val},
};

impl RootStyle for Dropdown<KeySelect> {
	fn root_style(&self) -> Style {
		Style {
			position_type: PositionType::Absolute,
			top: Val::Percent(100.),
			right: Val::Percent(0.),
			..default()
		}
	}
}

impl GetLayout for Dropdown<KeySelect> {
	fn layout(&self) -> Layout {
		Layout::SINGLE_COLUMN
	}
}