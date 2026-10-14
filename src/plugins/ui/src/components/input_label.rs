use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(Node = Self::node(), TextFont = Self::text_font(), TextLayout = Self::text_layout())]
pub struct InputLabel {
	pub key: HandSlot,
}

impl InputLabel {
	pub(crate) const FONT_SIZE: FontSize = FontSize::Px(15.);

	fn node() -> Node {
		Node {
			margin: UiRect::all(Val::Auto),
			align_content: AlignContent::Center,
			..default()
		}
	}

	fn text_font() -> TextFont {
		TextFont {
			font_size: Self::FONT_SIZE,
			..default()
		}
	}

	fn text_layout() -> TextLayout {
		TextLayout::justify(Justify::Center)
	}
}
