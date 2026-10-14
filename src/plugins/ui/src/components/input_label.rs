use bevy::prelude::*;
use common::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(Node = Self::node(), TextLayout = Self::text_layout())]
pub struct InputLabel {
	pub key: HandSlot,
}

impl InputLabel {
	fn node() -> Node {
		Node::default()
	}

	fn text_layout() -> TextLayout {
		TextLayout::justify(Justify::Center)
	}
}
