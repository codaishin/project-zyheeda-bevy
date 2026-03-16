use crate::{
	components::menu_background::MenuBackground,
	traits::{LoadUi, insert_ui_content::InsertUiContent},
};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::traits::{
	handles_localization::{Localize, LocalizeToken},
	thread_safe::ThreadSafe,
};

#[derive(Component, Debug, PartialEq)]
#[require(MenuBackground)]
pub(crate) struct PauseMenu;

impl LoadUi<AssetServer> for PauseMenu {
	fn load_ui(_: &mut AssetServer) -> Self {
		Self
	}
}

impl InsertUiContent for PauseMenu {
	fn insert_ui_content<TLocalization>(
		&self,
		localization: &TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) where
		TLocalization: Localize + ThreadSafe,
	{
		parent.spawn((
			Text::from(localization.localize_token("paused").or_token()),
			TextFont {
				font_size: 40.0,
				..default()
			},
		));
	}
}
