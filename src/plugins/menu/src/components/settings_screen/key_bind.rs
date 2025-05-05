use crate::traits::{
	colors::DEFAULT_PANEL_COLORS,
	insert_ui_content::InsertUiContent,
	ui_traits::{GetBackgroundColor, GetNode},
};
use bevy::prelude::*;
use common::{
	tools::keys::{Key, user_input::UserInput},
	traits::{
		handles_localization::{LocalizeToken, Token},
		thread_safe::ThreadSafe,
	},
};

#[derive(Component, Debug, PartialEq)]
#[require(Node(Self::node), BackgroundColor(Self::background_color))]
pub(crate) struct KeyBind<TUserInput>(pub(crate) TUserInput)
where
	Self: GetBackgroundColor + GetNode;

impl<TUserInput> KeyBind<TUserInput>
where
	Self: GetBackgroundColor + GetNode,
{
	fn node_base() -> Node {
		Node {
			width: Val::Px(200.0),
			height: Val::Px(20.0),
			margin: UiRect::all(Val::Px(2.0)),
			padding: UiRect::all(Val::Px(2.0)),
			align_items: AlignItems::Center,
			..default()
		}
	}
}

impl<TUserInput> InsertUiContent for KeyBind<TUserInput>
where
	Self: GetBackgroundColor + GetNode,
	TUserInput: Into<Token> + Copy + 'static,
{
	fn insert_ui_content<TLocalization>(
		&self,
		localization: &mut TLocalization,
		parent: &mut ChildBuilder,
	) where
		TLocalization: LocalizeToken + ThreadSafe,
	{
		parent.spawn((
			Text::from(localization.localize_token(self.0).or_token()),
			TextFont {
				font_size: 15.0,
				..default()
			},
			TextColor::from(DEFAULT_PANEL_COLORS.text),
		));
	}
}

type KeyBindAction = KeyBind<Key>;

impl GetNode for KeyBindAction {
	fn node() -> Node {
		let mut node = Self::node_base();
		node.justify_content = JustifyContent::End;
		node
	}
}

impl GetBackgroundColor for KeyBindAction {
	fn background_color() -> Color {
		DEFAULT_PANEL_COLORS.empty
	}
}

type KeyBindInput = KeyBind<UserInput>;

impl GetNode for KeyBindInput {
	fn node() -> Node {
		let mut node = Self::node_base();
		node.justify_content = JustifyContent::Center;
		node
	}
}

impl GetBackgroundColor for KeyBindInput {
	fn background_color() -> Color {
		DEFAULT_PANEL_COLORS.filled
	}
}
