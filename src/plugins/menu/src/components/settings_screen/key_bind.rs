mod action_key;
mod user_input;

use crate::traits::{
	colors::DEFAULT_PANEL_COLORS,
	insert_ui_content::InsertUiContent,
	ui_traits::{GetBackgroundColor, GetNode},
};
use bevy::prelude::*;
use common::traits::{
	handles_localization::{LocalizeToken, Token},
	thread_safe::ThreadSafe,
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
			width: Val::Percent(50.),
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
