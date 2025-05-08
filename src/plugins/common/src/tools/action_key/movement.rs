use super::{ActionKey, IsNot, user_input::UserInput};
use crate::traits::{
	handles_localization::Token,
	iteration::{Iter, IterFinite},
};
use bevy::input::{keyboard::KeyCode, mouse::MouseButton};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Copy, Eq, Hash, PartialEq, Debug, Serialize, Deserialize)]
pub enum MovementKey {
	#[default]
	Forward,
	Backward,
	Left,
	Right,
	Pointer,
}

impl IterFinite for MovementKey {
	fn iterator() -> Iter<Self> {
		Iter(Some(Self::default()))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			MovementKey::Forward => Some(MovementKey::Backward),
			MovementKey::Backward => Some(MovementKey::Left),
			MovementKey::Left => Some(MovementKey::Right),
			MovementKey::Right => Some(MovementKey::Pointer),
			MovementKey::Pointer => None,
		}
	}
}

impl From<MovementKey> for UserInput {
	fn from(value: MovementKey) -> Self {
		match value {
			MovementKey::Forward => Self::from(KeyCode::KeyW),
			MovementKey::Backward => Self::from(KeyCode::KeyS),
			MovementKey::Left => Self::from(KeyCode::KeyA),
			MovementKey::Right => Self::from(KeyCode::KeyD),
			MovementKey::Pointer => Self::from(MouseButton::Left),
		}
	}
}

impl From<MovementKey> for ActionKey {
	fn from(key: MovementKey) -> Self {
		Self::Movement(key)
	}
}

impl TryFrom<ActionKey> for MovementKey {
	type Error = IsNot<MovementKey>;

	fn try_from(key: ActionKey) -> Result<Self, Self::Error> {
		match key {
			ActionKey::Movement(key) => Ok(key),
			_ => Err(IsNot::key()),
		}
	}
}

impl From<MovementKey> for Token {
	fn from(value: MovementKey) -> Self {
		match value {
			MovementKey::Forward => Token::from("movement-key-forward"),
			MovementKey::Backward => Token::from("movement-key-backward"),
			MovementKey::Left => Token::from("movement-key-left"),
			MovementKey::Right => Token::from("movement-key-right"),
			MovementKey::Pointer => Token::from("movement-key-pointer"),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_all_keys() {
		assert_eq!(
			vec![
				MovementKey::Forward,
				MovementKey::Backward,
				MovementKey::Left,
				MovementKey::Right,
				MovementKey::Pointer,
			],
			MovementKey::iterator().take(5).collect::<Vec<_>>()
		);
	}
}
