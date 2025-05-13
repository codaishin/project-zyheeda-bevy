use bevy::prelude::*;

#[derive(Component, Debug, PartialEq)]
#[require(Node(Self::node), TextFont(Self::text_font))]
pub struct InputLabel<TKey> {
	pub key: TKey,
}

impl<TKey> InputLabel<TKey> {
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
