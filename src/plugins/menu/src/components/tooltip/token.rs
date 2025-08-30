use super::{Tooltip, TooltipUiConfig};
use crate::traits::{colors::PanelColors, insert_ui_content::InsertUiContent};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::traits::handles_localization::{Localize, Token, localized::Localized};

impl TooltipUiConfig for Token {
	fn node() -> Node {
		Node {
			top: Val::Px(-25.0),
			padding: UiRect::all(Val::Px(5.0)),
			..default()
		}
	}

	fn background_color() -> BackgroundColor {
		BackgroundColor(PanelColors::DEFAULT.filled.text)
	}
}

impl InsertUiContent for Tooltip<Token> {
	fn insert_ui_content<TLocalization>(
		&self,
		localize: &TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) where
		TLocalization: Localize,
	{
		let localized = localize.localize(&self.0).or_token();

		parent.spawn((
			Text::from(localized),
			TextFont {
				font_size: 20.0,
				..default()
			},
			TextColor(PanelColors::DEFAULT.filled.background),
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
		BackgroundColor(PanelColors::DEFAULT.filled.text)
	}
}

impl InsertUiContent for Tooltip<Localized> {
	fn insert_ui_content<TLocalization>(
		&self,
		_: &TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) {
		parent.spawn((
			Text::from(self.0.clone()),
			TextFont {
				font_size: 20.0,
				..default()
			},
			TextColor(PanelColors::DEFAULT.filled.background),
		));
	}
}
