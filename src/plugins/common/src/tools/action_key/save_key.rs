use crate::{
	tools::action_key::user_input::UserInput,
	traits::{
		handles_input::InvalidUserInput,
		handles_localization::Token,
		iteration::{FiniteIter, IterFinite},
	},
};
use bevy::input::keyboard::KeyCode;
use macros::serde_model;

#[serde_model]
#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum SaveKey {
	QuickSave,
	QuickLoad,
}

impl From<SaveKey> for Token {
	fn from(save_key: SaveKey) -> Self {
		match save_key {
			SaveKey::QuickSave => Self::from("save-quick-save"),
			SaveKey::QuickLoad => Self::from("save-quick-load"),
		}
	}
}

impl From<SaveKey> for UserInput {
	fn from(value: SaveKey) -> Self {
		match value {
			SaveKey::QuickSave => Self::KeyCode(KeyCode::F5),
			SaveKey::QuickLoad => Self::KeyCode(KeyCode::F9),
		}
	}
}

impl IterFinite for SaveKey {
	fn iterator() -> FiniteIter<Self> {
		FiniteIter(Some(Self::QuickSave))
	}

	fn next(FiniteIter(current): &FiniteIter<Self>) -> Option<Self> {
		match current.as_ref()? {
			Self::QuickSave => Some(Self::QuickLoad),
			Self::QuickLoad => None,
		}
	}
}

impl InvalidUserInput for SaveKey {
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
			vec![SaveKey::QuickSave, SaveKey::QuickLoad],
			SaveKey::iterator().take(100).collect::<Vec<_>>()
		);
	}
}
