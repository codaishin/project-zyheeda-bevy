use super::KeyBind;
use crate::traits::{
	colors::DEFAULT_PANEL_COLORS,
	ui_traits::{GetBackgroundColor, GetNode},
};
use bevy::prelude::*;
use common::tools::action_key::ActionKey;

impl GetNode for KeyBind<ActionKey> {
	fn node() -> Node {
		let mut node = Self::node_base();
		node.justify_content = JustifyContent::End;
		node
	}
}

impl GetBackgroundColor for KeyBind<ActionKey> {
	fn background_color() -> Color {
		DEFAULT_PANEL_COLORS.empty
	}
}
