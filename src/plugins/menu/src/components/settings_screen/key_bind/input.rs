use super::KeyBind;
use crate::traits::{
	colors::PanelColors,
	ui_traits::{GetBackgroundColor, GetNode},
};
use bevy::prelude::*;
use common::{tools::action_key::user_input::UserInput, traits::handles_localization::Token};

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Input<TAction> {
	pub(crate) action: TAction,
	pub(crate) input: UserInput,
}

impl<TAction> GetNode for KeyBind<Input<TAction>> {
	fn node() -> Node {
		let mut node = Self::node_base();
		node.justify_content = JustifyContent::Center;
		node
	}
}

impl<TAction> GetBackgroundColor for KeyBind<Input<TAction>> {
	fn background_color() -> Color {
		PanelColors::DEFAULT.filled.background
	}
}

impl<TAction> From<Input<TAction>> for Token {
	fn from(Input { input, .. }: Input<TAction>) -> Self {
		input.into()
	}
}
