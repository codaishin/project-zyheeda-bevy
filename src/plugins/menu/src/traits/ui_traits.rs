use bevy::prelude::*;

pub(crate) trait GetNode {
	fn node() -> Node;
}

pub(crate) trait GetBackgroundColor {
	fn background_color() -> Color;
}
