pub(crate) mod key_bind;

use super::menu_background::MenuBackground;
use crate::{
	LoadUi,
	traits::{
		colors::PanelColors,
		insert_ui_content::InsertUiContent,
		update_key_bindings::UpdateKeyBindings,
	},
};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::{
	states::menu_state::MenuState,
	tools::action_key::{
		ActionKey,
		camera_key::CameraKey,
		miscellaneous::Miscellaneous,
		movement::MovementKey,
		save_key::SaveKey,
		slot::HandSlot,
		targeting::TerrainTargeting,
		user_input::UserInput,
	},
	traits::{
		handles_input::GetAllInputs,
		handles_localization::{Localize, LocalizeToken, Token},
		iteration::IterFinite,
	},
};
use key_bind::{KeyBind, action::Action, input::Input};
use std::collections::HashMap;

#[derive(Component, Debug, PartialEq, Default)]
#[require(MenuBackground)]
pub(crate) struct SettingsScreen {
	key_bindings: HashMap<ActionKey, UserInput>,
}

impl SettingsScreen {
	fn add_title(
		parent: &mut RelatedSpawnerCommands<ChildOf>,
		localize: &impl Localize,
		title: impl Into<Token>,
	) {
		parent.spawn((
			Text::from(localize.localize_token(title).or_token()),
			TextFont {
				font_size: FontSize::Px(40.0),
				..default()
			},
		));
	}

	fn add_section_title(
		parent: &mut RelatedSpawnerCommands<ChildOf>,
		localize: &impl Localize,
		title: impl Into<Token>,
	) {
		parent
			.spawn(Node {
				justify_content: JustifyContent::Start,
				..default()
			})
			.with_child((
				Text::from(localize.localize_token(title).or_token()),
				TextFont {
					font_size: FontSize::Px(20.0),
					..default()
				},
			));
	}

	fn add_section<T>(
		&self,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
		keys: impl IntoIterator<Item = T>,
		localize: &impl Localize,
		title: impl Into<Token>,
	) where
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
				BackgroundColor(PanelColors::DEFAULT.empty.background),
			))
			.with_children(|parent| {
				Self::add_section_title(parent, localize, title);
				self.add_key_bindings(parent, keys);
			});
	}

	fn add_key_bindings<T>(
		&self,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
		keys: impl IntoIterator<Item = T>,
	) where
		ActionKey: From<T>,
	{
		for (action, input) in self.keys(keys) {
			Self::add_key_row(parent, action, input);
		}
	}

	fn keys<T>(
		&self,
		keys: impl IntoIterator<Item = T>,
	) -> impl Iterator<Item = (ActionKey, UserInput)>
	where
		ActionKey: From<T>,
	{
		keys.into_iter()
			.map(ActionKey::from)
			.filter_map(|key| Some((key, *self.key_bindings.get(&key)?)))
	}

	fn add_key_row(
		parent: &mut RelatedSpawnerCommands<ChildOf>,
		action: ActionKey,
		input: UserInput,
	) {
		parent
			.spawn(Node {
				flex_direction: FlexDirection::Row,
				..default()
			})
			.with_children(|parent| {
				parent.spawn(KeyBind(Action(action)));
				parent.spawn(KeyBind(Input { action, input }));
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
		localize: &TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) where
		TLocalization: Localize,
	{
		parent
			.spawn(Node {
				flex_direction: FlexDirection::Column,
				align_items: AlignItems::Center,
				..default()
			})
			.with_children(|parent| {
				Self::add_title(parent, localize, "key-bindings");
				self.add_section(parent, HandSlot::iterator(), localize, "key-bindings-slots");
				self.add_section(
					parent,
					TerrainTargeting::iterator(),
					localize,
					"key-bindings-targeting",
				);
				self.add_section(
					parent,
					MovementKey::iterator(),
					localize,
					"key-bindings-movement",
				);
				self.add_section(
					parent,
					Miscellaneous::iterator()
						.map(ActionKey::from)
						.chain(MenuState::iterator().map(ActionKey::from)),
					localize,
					"key-bindings-miscellaneous",
				);
				self.add_section(
					parent,
					CameraKey::iterator(),
					localize,
					"key-bindings-camera",
				);
				self.add_section(
					parent,
					SaveKey::iterator(),
					localize,
					"key-bindings-savegame",
				);
			});
	}
}

impl UpdateKeyBindings for SettingsScreen {
	fn update_key_bindings<TInput>(&mut self, input: &TInput)
	where
		TInput: GetAllInputs,
	{
		self.key_bindings = input.get_all_inputs().collect()
	}
}
