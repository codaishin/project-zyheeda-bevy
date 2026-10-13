use crate::traits::handles_localization::TokenItem;
use bevy::prelude::*;

impl From<MouseButton> for TokenItem<String> {
	fn from(value: MouseButton) -> Self {
		match value {
			MouseButton::Left => Self::Key(String::from("mouse-button-left")),
			MouseButton::Right => Self::Key(String::from("mouse-button-right")),
			MouseButton::Middle => Self::Key(String::from("mouse-button-middle")),
			MouseButton::Back => Self::Key(String::from("mouse-button-back")),
			MouseButton::Forward => Self::Key(String::from("mouse-button-forward")),
			MouseButton::Other(index) => Self::Key(format!("mouse-button-other-{index}")),
		}
	}
}
