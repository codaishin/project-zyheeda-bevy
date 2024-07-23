use super::Dropdown;
use crate::{
	components::{combo_overview::ComboOverview, tooltip::Tooltip},
	tools::{Layout, SkillSelect},
	traits::{
		get_node::GetNode,
		instantiate_content_on::InstantiateContentOn,
		GetLayout,
		RootStyle,
	},
};
use bevy::{
	prelude::{default, ChildBuilder, NodeBundle},
	ui::{PositionType, Style, Val},
};

impl GetNode for SkillSelect {
	fn node(&self) -> NodeBundle {
		NodeBundle::default()
	}
}

impl InstantiateContentOn for SkillSelect {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		parent.spawn((
			ComboOverview::skill_button_bundle(self.0.icon.clone().unwrap_or_default()),
			Tooltip(self.clone()),
		));
	}
}

impl RootStyle for Dropdown<SkillSelect> {
	fn root_style(&self) -> Style {
		Style {
			position_type: PositionType::Absolute,
			top: Val::Percent(100.),
			right: Val::Percent(0.),
			..default()
		}
	}
}

impl GetLayout for Dropdown<SkillSelect> {
	fn layout(&self) -> Layout {
		Layout::SINGLE_COLUMN
	}
}
