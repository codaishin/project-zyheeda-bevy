pub(crate) mod action;
pub(crate) mod input;
pub(crate) mod rebinding;

use crate::traits::{
	colors::PanelColors,
	insert_ui_content::InsertUiContent,
	ui_traits::{GetBackgroundColor, GetNode},
};
use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};
use common::traits::{
	handles_localization::{LocalizeToken, Token},
	thread_safe::ThreadSafe,
};

#[derive(Component, Debug, PartialEq, Clone, Copy)]
#[require(Node = Self::node(), BackgroundColor = Self::background_color())]
pub(crate) struct KeyBind<T>(pub(crate) T)
where
	Self: GetBackgroundColor + GetNode;

impl<T> KeyBind<T>
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

impl<T> InsertUiContent for KeyBind<T>
where
	Self: GetBackgroundColor + GetNode,
	T: Into<Token> + Copy + 'static,
{
	fn insert_ui_content<TLocalization>(
		&self,
		localization: &mut TLocalization,
		parent: &mut RelatedSpawnerCommands<ChildOf>,
	) where
		TLocalization: LocalizeToken + ThreadSafe,
	{
		parent.spawn((
			Text::from(localization.localize_token(self.0).or_token()),
			TextFont {
				font_size: 15.0,
				..default()
			},
			TextColor::from(PanelColors::DEFAULT.filled.text),
		));
	}
}
