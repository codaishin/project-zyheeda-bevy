use super::KeyBind;
use crate::traits::{
	colors::PanelColors,
	ui_traits::{GetBackgroundColor, GetNode},
};
use bevy::prelude::*;
use common::traits::handles_localization::Token;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Action<T>(pub(crate) T);

impl<T> GetNode for KeyBind<Action<T>> {
	fn node() -> Node {
		let mut node = Self::node_base();
		node.justify_content = JustifyContent::End;
		node
	}
}

impl<T> GetBackgroundColor for KeyBind<Action<T>> {
	fn background_color() -> Color {
		PanelColors::DEFAULT.empty
	}
}

impl<T> From<Action<T>> for Token
where
	T: Into<Token>,
{
	fn from(Action(action): Action<T>) -> Self {
		action.into()
	}
}
