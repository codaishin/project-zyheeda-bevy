use super::Tooltip;
use crate::traits::{
	colors::DEFAULT_PANEL_COLORS,
	get_node::GetNode,
	instantiate_content_on::InstantiateContentOn,
};
use bevy::{
	prelude::{default, ChildBuilder, NodeBundle, TextBundle},
	text::TextStyle,
	ui::{Style, UiRect, Val, ZIndex},
};
use skills::skills::Skill;

impl GetNode for Tooltip<Skill> {
	fn node(&self) -> NodeBundle {
		NodeBundle {
			style: Style {
				top: Val::Px(-25.0),
				padding: UiRect::all(Val::Px(5.0)),
				..default()
			},
			background_color: DEFAULT_PANEL_COLORS.text.into(),
			z_index: ZIndex::Global(1),
			..default()
		}
	}
}

impl InstantiateContentOn for Tooltip<Skill> {
	fn instantiate_content_on(&self, parent: &mut ChildBuilder) {
		parent.spawn(TextBundle::from_section(
			self.0.name.clone(),
			TextStyle {
				font_size: 20.0,
				color: DEFAULT_PANEL_COLORS.filled,
				..default()
			},
		));
	}
}
