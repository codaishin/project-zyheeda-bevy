use super::Dropdown;
use crate::{
	components::key_select::{AppendSkill, KeySelect, ReKeySkill},
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
			top: Val::Percent(100.),
			right: Val::Percent(0.),
			..default()
		}
	}
}

impl RootStyle for Dropdown<KeySelect<AppendSkill>> {
	fn root_style(&self) -> Style {
		Style {
			position_type: PositionType::Absolute,
			top: Val::Percent(0.),
			left: Val::Percent(100.),
			..default()
		}
	}
}

impl<TExtra> GetLayout for Dropdown<KeySelect<TExtra>> {
	fn layout(&self) -> Layout {
		Layout::SINGLE_COLUMN
	}
}
