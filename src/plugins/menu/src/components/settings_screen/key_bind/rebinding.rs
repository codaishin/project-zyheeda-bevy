use super::KeyBind;
use crate::{
	Input,
	traits::ui_traits::{GetBackgroundColor, GetNode},
};
use bevy::prelude::*;
use common::traits::handles_localization::Token;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Rebinding<TAction, TInput>(pub(crate) Input<TAction, TInput>);

impl<TAction, TInput> GetNode for KeyBind<Rebinding<TAction, TInput>> {
	fn node() -> Node {
		KeyBind::<Input<TAction, TInput>>::node()
	}
}

impl<TAction, TInput> GetBackgroundColor for KeyBind<Rebinding<TAction, TInput>> {
	fn background_color() -> Color {
		KeyBind::<Input<TAction, TInput>>::background_color()
	}
}

impl<TAction, TInput> From<Rebinding<TAction, TInput>> for Token {
	fn from(_: Rebinding<TAction, TInput>) -> Self {
		Self::from("rebind-text-prompt")
	}
}
