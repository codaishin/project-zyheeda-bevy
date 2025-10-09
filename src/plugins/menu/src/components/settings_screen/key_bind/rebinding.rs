use super::KeyBind;
use crate::{
	Input,
	traits::ui_traits::{GetBackgroundColor, GetNode},
};
use bevy::prelude::*;
use common::traits::handles_localization::Token;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) struct Rebinding<TAction>(pub(crate) Input<TAction>);

impl<TAction> GetNode for KeyBind<Rebinding<TAction>> {
	fn node() -> Node {
		KeyBind::<Input<TAction>>::node()
	}
}

impl<TAction> GetBackgroundColor for KeyBind<Rebinding<TAction>> {
	fn background_color() -> Color {
		KeyBind::<Input<TAction>>::background_color()
	}
}

impl<TAction> From<Rebinding<TAction>> for Token {
	fn from(_: Rebinding<TAction>) -> Self {
		Self::from("rebind-text-prompt")
	}
}
