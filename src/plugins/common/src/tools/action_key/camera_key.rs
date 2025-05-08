use super::{ActionKey, IsNot, user_input::UserInput};
use crate::traits::{
	handles_localization::Token,
	iteration::{Iter, IterFinite},
};
use bevy::input::mouse::MouseButton;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub enum CameraKey {
	Rotate,
}

impl From<CameraKey> for Token {
	fn from(camera_key: CameraKey) -> Self {
		match camera_key {
			CameraKey::Rotate => Self::from("camera-key-rotate"),
		}
	}
}

impl From<CameraKey> for ActionKey {
	fn from(camera_key: CameraKey) -> Self {
		Self::Camera(camera_key)
	}
}

impl From<CameraKey> for UserInput {
	fn from(value: CameraKey) -> Self {
		match value {
			CameraKey::Rotate => UserInput::MouseButton(MouseButton::Right),
		}
	}
}

impl TryFrom<ActionKey> for CameraKey {
	type Error = IsNot<CameraKey>;

	fn try_from(value: ActionKey) -> Result<Self, Self::Error> {
		match value {
			ActionKey::Camera(camera_key) => Ok(camera_key),
			_ => Err(IsNot::key()),
		}
	}
}

impl IterFinite for CameraKey {
	fn iterator() -> Iter<Self> {
		Iter(Some(Self::Rotate))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match &current.0? {
			CameraKey::Rotate => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iterate() {
		assert_eq!(
			vec![CameraKey::Rotate],
			CameraKey::iterator().take(100).collect::<Vec<_>>()
		);
	}
}
