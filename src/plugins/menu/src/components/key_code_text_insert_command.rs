use bevy::prelude::*;

#[derive(Component, Debug, Default)]
#[require(Text)]
pub(crate) struct UserInputTextInsertCommand<TKey> {
	pub(crate) key: TKey,
	pub(crate) font: TextFont,
	pub(crate) color: TextColor,
	pub(crate) layout: TextLayout,
}

impl<TKey: PartialEq> PartialEq for UserInputTextInsertCommand<TKey> {
	fn eq(&self, other: &Self) -> bool {
		if self.key != other.key {
			return false;
		}

		let TextColor(s_color) = &self.color;
		let TextColor(o_color) = &other.color;

		self.font.font == other.font.font
			&& self.font.font_size == other.font.font_size
			&& self.font.font_smoothing == other.font.font_smoothing
			&& s_color == o_color
			&& self.layout.justify == other.layout.justify
			&& self.layout.linebreak == other.layout.linebreak
	}
}
