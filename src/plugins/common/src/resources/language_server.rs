use crate::traits::get_ui_text::{English, GetUiText, GetUiTextFor, Japanese, UIText};
use bevy::ecs::system::Resource;

#[derive(Default, Resource)]
pub enum LanguageServer {
	#[default]
	EN,
	JP,
}

impl<TKey> GetUiTextFor<TKey> for LanguageServer {
	fn ui_text_for(&self, value: &TKey) -> UIText
	where
		Japanese: GetUiText<TKey>,
		English: GetUiText<TKey>,
	{
		match self {
			LanguageServer::EN => English::ui_text(value),
			LanguageServer::JP => Japanese::ui_text(value),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::get_ui_text::{GetUiText, UIText};

	struct _Key(&'static str);

	impl GetUiText<_Key> for English {
		fn ui_text(value: &_Key) -> UIText {
			UIText::from(value.0)
		}
	}

	#[test]
	fn test_english() {
		assert_eq!(
			UIText::from("Hello"),
			LanguageServer::EN.ui_text_for(&_Key("Hello")),
		)
	}

	impl GetUiText<_Key> for Japanese {
		fn ui_text(value: &_Key) -> UIText {
			UIText::from(value.0)
		}
	}

	#[test]
	fn test_japanese() {
		assert_eq!(
			UIText::from("Hello"),
			LanguageServer::JP.ui_text_for(&_Key("Hello")),
		);
	}
}
