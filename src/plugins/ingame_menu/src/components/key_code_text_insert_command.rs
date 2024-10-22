use bevy::{
	prelude::{Bundle, Component, TextBundle},
	text::TextStyle,
};
use skills::slot_key::SlotKey;

#[derive(Bundle, Debug, Default)]
pub(crate) struct KeyCodeTextInsertCommandBundle {
	insert_command: KeyCodeTextInsertCommand<SlotKey>,
	text_bundle: TextBundle,
}

impl KeyCodeTextInsertCommandBundle {
	pub(crate) fn new(key: SlotKey, text_style: TextStyle) -> Self {
		Self {
			insert_command: KeyCodeTextInsertCommand { key, text_style },
			text_bundle: TextBundle::default(),
		}
	}
}

#[derive(Component, Debug, Default)]
pub(crate) struct KeyCodeTextInsertCommand<TKey> {
	key: TKey,
	text_style: TextStyle,
}

impl<TKey> KeyCodeTextInsertCommand<TKey> {
	#[cfg(test)]
	pub(crate) fn new(key: TKey, text_style: TextStyle) -> Self {
		Self { key, text_style }
	}

	pub(crate) fn key(&self) -> &TKey {
		&self.key
	}

	pub(crate) fn text_style(&self) -> &TextStyle {
		&self.text_style
	}
}

impl<TKey: PartialEq> PartialEq for KeyCodeTextInsertCommand<TKey> {
	fn eq(&self, other: &Self) -> bool {
		self.key == other.key
			&& self.text_style.color == other.text_style.color
			&& self.text_style.font == other.text_style.font
			&& self.text_style.font_size == other.text_style.font_size
	}
}
