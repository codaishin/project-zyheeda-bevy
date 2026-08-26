use super::user_input::UserInput;
use crate::traits::{
	handles_input::InvalidUserInput,
	handles_localization::Token,
	iteration::{FiniteIter, IterFinite},
};
use bevy::input::mouse::MouseButton;
use macros::serde_model;

#[serde_model]
#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
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

impl From<CameraKey> for UserInput {
	fn from(value: CameraKey) -> Self {
		match value {
			CameraKey::Rotate => UserInput::MouseButton(MouseButton::Right),
		}
	}
}

impl IterFinite for CameraKey {
	fn iterator() -> FiniteIter<Self> {
		FiniteIter(Some(Self::Rotate))
	}

	fn next(current: &FiniteIter<Self>) -> Option<Self> {
		match &current.0? {
			CameraKey::Rotate => None,
		}
	}
}

impl InvalidUserInput for CameraKey {
	fn invalid_input(&self) -> &[UserInput] {
		const { &[] }
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
