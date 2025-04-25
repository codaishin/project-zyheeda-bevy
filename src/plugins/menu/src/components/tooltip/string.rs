use super::{Tooltip, TooltipUiConfig};
use crate::traits::{colors::DEFAULT_PANEL_COLORS, insert_ui_content::InsertUiContent};
use bevy::prelude::*;
use common::traits::handles_localization::{LocalizeToken, localized::Localized};

impl TooltipUiConfig for &'static str {
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

impl InsertUiContent for Tooltip<&'static str> {
	fn insert_ui_content<TLocalization>(
		&self,
		localize: &mut TLocalization,
		parent: &mut ChildBuilder,
	) where
		TLocalization: LocalizeToken,
	{
		let localized = localize.localize_token(self.0).or_token();

		parent.spawn((
			Text::from(localized),
			TextFont {
				font_size: 20.0,
				..default()
			},
			TextColor(DEFAULT_PANEL_COLORS.filled),
		));
	}
}

impl TooltipUiConfig for String {
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

impl InsertUiContent for Tooltip<String> {
	fn insert_ui_content<TLocalization>(
		&self,
		localize: &mut TLocalization,
		parent: &mut ChildBuilder,
	) where
		TLocalization: LocalizeToken,
	{
		let localized = localize.localize_token(self.0.clone()).or_token();

		parent.spawn((
			Text::from(localized),
			TextFont {
				font_size: 20.0,
				..default()
			},
			TextColor(DEFAULT_PANEL_COLORS.filled),
		));
	}
}

impl TooltipUiConfig for Localized {
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

impl InsertUiContent for Tooltip<Localized> {
	fn insert_ui_content<TLocalization>(&self, _: &mut TLocalization, parent: &mut ChildBuilder) {
		parent.spawn((
			Text::from(self.0.clone()),
			TextFont {
				font_size: 20.0,
				..default()
			},
			TextColor(DEFAULT_PANEL_COLORS.filled),
		));
	}
}
