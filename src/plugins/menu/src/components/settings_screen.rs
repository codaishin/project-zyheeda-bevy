pub(crate) mod key_bind;

use super::menu_background::MenuBackground;
use crate::{
	LoadUi,
	traits::{
		colors::DEFAULT_PANEL_COLORS,
		insert_ui_content::InsertUiContent,
		update_key_bindings::UpdateKeyBindings,
	},
};
use bevy::prelude::*;
use common::{
	states::menu_state::MenuState,
	tools::action_key::{
		ActionKey,
		camera_key::CameraKey,
		movement::MovementKey,
		slot::SlotKey,
		user_input::UserInput,
	},
	traits::{
		handles_localization::{LocalizeToken, Token},
		iterate::Iterate,
		iteration::IterFinite,
		thread_safe::ThreadSafe,
	},
};
use key_bind::KeyBind;
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq, Default)]
#[require(MenuBackground)]
pub(crate) struct SettingsScreen {
	key_bindings: HashMap<ActionKey, UserInput>,
}

impl SettingsScreen {
	fn add_title(
		parent: &mut ChildBuilder,
		localize: &mut (impl LocalizeToken + ThreadSafe),
		title: (impl Into<Token> + 'static),
	) {
		parent.spawn((
			Text::new(localize.localize_token(title).or_token()),
			TextFont {
				font_size: 40.0,
				..default()
			},
			TextColor(DEFAULT_PANEL_COLORS.text),
		));
	}

	fn add_section_title(
		parent: &mut ChildBuilder,
		localize: &mut (impl LocalizeToken + ThreadSafe),
		title: (impl Into<Token> + 'static),
	) {
		parent
			.spawn(Node {
				justify_content: JustifyContent::Start,
				..default()
			})
			.with_child((
				Text::new(localize.localize_token(title).or_token()),
				TextFont {
					font_size: 20.0,
					..default()
				},
				TextColor(DEFAULT_PANEL_COLORS.text),
			));
	}

	fn add_section<T>(
		&self,
		parent: &mut ChildBuilder,
		localize: &mut (impl LocalizeToken + ThreadSafe),
		title: (impl Into<Token> + 'static),
	) where
		T: IterFinite,
		ActionKey: From<T>,
	{
		parent
			.spawn((
				Node {
					width: Val::Px(400.),
					justify_content: JustifyContent::Center,
					flex_direction: FlexDirection::Column,
					padding: UiRect::all(Val::Px(2.)),
					margin: UiRect::all(Val::Px(2.)),
					..default()
				},
				BackgroundColor(DEFAULT_PANEL_COLORS.empty),
			))
			.with_children(|parent| {
				Self::add_section_title(parent, localize, title);
				self.add_key_bindings::<T>(parent);
			});
	}

	fn add_key_bindings<T>(&self, parent: &mut ChildBuilder)
	where
		T: IterFinite,
		ActionKey: From<T>,
	{
		for (key, user_input) in self.keys::<T>() {
			Self::add_key_row(parent, key, user_input);
		}
	}

	fn keys<T>(&self) -> impl Iterator<Item = (ActionKey, UserInput)>
	where
		T: IterFinite,
		ActionKey: From<T>,
	{
		T::iterator()
			.map(ActionKey::from)
			.filter_map(|key| Some((key, *self.key_bindings.get(&key)?)))
	}

	fn add_key_row(parent: &mut ChildBuilder, key: ActionKey, user_input: UserInput) {
		parent
			.spawn(Node {
				flex_direction: FlexDirection::Row,
				..default()
			})
			.with_children(|parent| {
				parent.spawn(KeyBind(key));
				parent.spawn(KeyBind(user_input));
			});
	}
}

impl LoadUi<AssetServer> for SettingsScreen {
	fn load_ui(_: &mut AssetServer) -> Self {
		Self::default()
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
				flex_direction: FlexDirection::Column,
				align_items: AlignItems::Center,
				..default()
			})
			.with_children(|parent| {
				Self::add_title(parent, localize, "key-bindings");
				self.add_section::<SlotKey>(parent, localize, "key-bindings-slots");
				self.add_section::<MovementKey>(parent, localize, "key-bindings-movement");
				self.add_section::<MenuState>(parent, localize, "key-bindings-menus");
				self.add_section::<CameraKey>(parent, localize, "key-bindings-camera");
			});
	}
}

impl UpdateKeyBindings<ActionKey, UserInput> for SettingsScreen {
	fn update_key_bindings<TKeyMap>(&mut self, map: &TKeyMap)
	where
		for<'a> TKeyMap: Iterate<'a, TItem = (&'a ActionKey, &'a UserInput)>,
	{
		self.key_bindings = map
			.iterate()
			.map(|(key, key_code)| (*key, *key_code))
			.collect()
	}
}
