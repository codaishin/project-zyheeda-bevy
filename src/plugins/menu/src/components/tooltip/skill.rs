use super::{Tooltip, TooltipUiConfig};
use crate::traits::{colors::DEFAULT_PANEL_COLORS, insert_ui_content::InsertUiContent};
use bevy::prelude::*;
use skills::skills::Skill;

impl TooltipUiConfig for Skill {
	fn node() -> Node {
		Node {
			top: Val::Px(-25.0),
			padding: UiRect::all(Val::Px(5.0)),
			..default()
		}
	}

	fn background_color() -> BackgroundColor {
		BackgroundColor(DEFAULT_PANEL_COLORS.text)
	}
}

impl InsertUiContent for Tooltip<Skill> {
	fn insert_ui_content(&self, parent: &mut ChildBuilder) {
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
