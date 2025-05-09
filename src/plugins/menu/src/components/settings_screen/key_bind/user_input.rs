use super::KeyBind;
use crate::traits::{
	colors::DEFAULT_PANEL_COLORS,
	ui_traits::{GetBackgroundColor, GetNode},
};
use bevy::prelude::*;
use common::tools::action_key::user_input::UserInput;

impl GetNode for KeyBind<UserInput> {
	fn node() -> Node {
		let mut node = Self::node_base();
		node.justify_content = JustifyContent::Center;
		node
	}
}

impl GetBackgroundColor for KeyBind<UserInput> {
	fn background_color() -> Color {
		DEFAULT_PANEL_COLORS.filled
	}
}
