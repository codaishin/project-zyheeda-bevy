use super::KeyBind;
use crate::traits::{
	colors::PanelColors,
	ui_traits::{GetBackgroundColor, GetNode},
};
use bevy::prelude::*;
use common::traits::handles_localization::Token;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Input<TAction, TInput> {
	pub(crate) action: TAction,
	pub(crate) input: TInput,
}

impl<TAction, TInput> GetNode for KeyBind<Input<TAction, TInput>> {
	fn node() -> Node {
		let mut node = Self::node_base();
		node.justify_content = JustifyContent::Center;
		node
	}
}

impl<TAction, TInput> GetBackgroundColor for KeyBind<Input<TAction, TInput>> {
	fn background_color() -> Color {
		PanelColors::DEFAULT.filled
	}
}

impl<TAction, TInput> From<Input<TAction, TInput>> for Token
where
	TInput: Into<Token>,
{
	fn from(Input { input, .. }: Input<TAction, TInput>) -> Self {
		input.into()
	}
}
