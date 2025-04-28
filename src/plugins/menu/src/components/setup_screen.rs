use crate::{LoadUi, traits::insert_ui_content::InsertUiContent};
use bevy::prelude::*;
use common::traits::{handles_localization::LocalizeToken, thread_safe::ThreadSafe};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct SetupScreen;

impl LoadUi<AssetServer> for SetupScreen {
	fn load_ui(_: &mut AssetServer) -> Self {
		Self
	}
}

impl InsertUiContent for SetupScreen {
	fn insert_ui_content<TLocalization>(&self, _: &mut TLocalization, _: &mut ChildBuilder)
	where
		TLocalization: LocalizeToken + ThreadSafe,
	{
	}
}
