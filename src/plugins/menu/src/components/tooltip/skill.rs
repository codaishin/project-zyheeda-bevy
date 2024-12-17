use super::Tooltip;
use crate::traits::{
	colors::DEFAULT_PANEL_COLORS,
	ui_components::GetUIComponents,
	update_children::UpdateChildren,
};
use bevy::prelude::*;
use skills::skills::Skill;

impl GetUIComponents for Tooltip<Skill> {
	fn ui_components(&self) -> (Node, BackgroundColor) {
		(
			Node {
				top: Val::Px(-25.0),
				padding: UiRect::all(Val::Px(5.0)),
				..default()
			},
			DEFAULT_PANEL_COLORS.text.into(),
		)
	}
}

impl UpdateChildren for Tooltip<Skill> {
	fn update_children(&self, parent: &mut ChildBuilder) {
		parent.spawn((
			Text::new(&self.0.name),
			TextFont {
				font_size: 20.0,
				..default()
			},
			TextColor(DEFAULT_PANEL_COLORS.filled),
		));
	}
}
