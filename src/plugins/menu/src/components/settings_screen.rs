use super::menu_background::MenuBackground;
use crate::{LoadUi, traits::insert_ui_content::InsertUiContent};
use bevy::prelude::*;
use common::traits::{handles_localization::LocalizeToken, thread_safe::ThreadSafe};

#[derive(Component, Debug, PartialEq)]
#[require(MenuBackground)]
pub(crate) struct SettingsScreen;

impl LoadUi<AssetServer> for SettingsScreen {
	fn load_ui(_: &mut AssetServer) -> Self {
		Self
	}
}

impl InsertUiContent for SettingsScreen {
	fn insert_ui_content<TLocalization>(&self, _: &mut TLocalization, _: &mut ChildBuilder)
	where
		TLocalization: LocalizeToken + ThreadSafe,
	{
	}
}
