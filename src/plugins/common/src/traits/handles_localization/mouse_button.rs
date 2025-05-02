use super::Token;
use bevy::prelude::*;

impl From<MouseButton> for Token {
	fn from(value: MouseButton) -> Self {
		match value {
			MouseButton::Left => Self::from("mouse-button-left"),
			MouseButton::Right => Self::from("mouse-button-right"),
			MouseButton::Middle => Self::from("mouse-button-middle"),
			MouseButton::Back => Self::from("mouse-button-back"),
			MouseButton::Forward => Self::from("mouse-button-forward"),
			MouseButton::Other(index) => Self(format!("mouse-button-other-{index}")),
		}
	}
}
