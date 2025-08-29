use bevy::prelude::*;
use common::tools::action_key::slot::PlayerSlot;

#[derive(Component, Debug, PartialEq)]
#[require(Node = Self::node(), TextFont = Self::text_font())]
pub struct InputLabel {
	pub key: PlayerSlot,
}

impl InputLabel {
	pub(crate) const FONT_SIZE: f32 = 15.;

	fn node() -> Node {
		Node {
			margin: UiRect::all(Val::Auto),
			..default()
		}
	}

	fn text_font() -> TextFont {
		TextFont {
			font_size: Self::FONT_SIZE,
			..default()
		}
	}
}
