use super::{English, GetUiText, Japanese, UIText};
use bevy::input::keyboard::KeyCode;

impl GetUiText<KeyCode> for English {
	fn ui_text(value: &KeyCode) -> UIText {
		match value {
			KeyCode::KeyQ => "Q".into(),
			KeyCode::KeyE => "E".into(),
			_ => UIText::Unmapped,
		}
	}
}

impl GetUiText<KeyCode> for Japanese {
	fn ui_text(value: &KeyCode) -> UIText {
		match value {
			KeyCode::KeyQ => "た".into(),
			KeyCode::KeyE => "い".into(),
			_ => UIText::Unmapped,
		}
	}
}
