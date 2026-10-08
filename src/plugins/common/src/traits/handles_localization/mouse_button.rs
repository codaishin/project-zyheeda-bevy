use crate::traits::handles_localization::TokenItem;
use bevy::prelude::*;

impl From<MouseButton> for TokenItem<String> {
	fn from(value: MouseButton) -> Self {
		match value {
			MouseButton::Left => TokenItem::Key(String::from("mouse-button-left")),
			MouseButton::Right => TokenItem::Key(String::from("mouse-button-right")),
			MouseButton::Middle => TokenItem::Key(String::from("mouse-button-middle")),
			MouseButton::Back => TokenItem::Key(String::from("mouse-button-back")),
			MouseButton::Forward => TokenItem::Key(String::from("mouse-button-forward")),
			MouseButton::Other(index) => TokenItem::Key(format!("mouse-button-other-{index}")),
		}
	}
}
