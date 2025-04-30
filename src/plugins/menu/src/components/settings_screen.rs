use super::menu_background::MenuBackground;
use crate::{
	LoadUi,
	traits::{colors::DEFAULT_PANEL_COLORS, insert_ui_content::InsertUiContent},
};
use bevy::prelude::*;
use common::traits::{
	handles_localization::{LocalizeToken, localized::Localized},
	thread_safe::ThreadSafe,
};

#[derive(Component, Debug, PartialEq)]
#[require(MenuBackground)]
pub(crate) struct SettingsScreen;

impl LoadUi<AssetServer> for SettingsScreen {
	fn load_ui(_: &mut AssetServer) -> Self {
		Self
	}
}

impl InsertUiContent for SettingsScreen {
	fn insert_ui_content<TLocalization>(
		&self,
		localize: &mut TLocalization,
		parent: &mut ChildBuilder,
	) where
		TLocalization: LocalizeToken + ThreadSafe,
	{
		parent
			.spawn(Node {
				flex_direction: FlexDirection::Row,
				align_items: AlignItems::Center,
				..default()
			})
			.with_children(|parent| {
				add_title(parent, localize.localize_token("key-bindings").or_token());
			});
	}
}

fn add_title(parent: &mut ChildBuilder, title: Localized) {
	parent.spawn((
		Text::new(title),
		TextFont {
			font_size: 40.0,
			..default()
		},
		TextColor(DEFAULT_PANEL_COLORS.text),
	));
}
