use super::user_input::UserInput;
use crate::{
	tools::action_key::user_input::combination::UserInputCombination,
	traits::{
		handles_input::InvalidUserInput,
		handles_localization::Token,
		iteration::{FiniteIter, IterFinite},
	},
};
use macros::serde_model;

#[serde_model]
#[derive(Clone, Copy, Eq, Hash, PartialEq, Debug)]
pub enum DebugKey {
	RotateColliderDebug,
}

impl From<DebugKey> for UserInput {
	fn from(value: DebugKey) -> Self {
		match value {
			DebugKey::RotateColliderDebug => {
				UserInput::Combined(UserInputCombination::ROTATE_DEBUG_NAV_MESH)
			}
		}
	}
}

impl From<DebugKey> for Token {
	fn from(value: DebugKey) -> Self {
		match value {
			DebugKey::RotateColliderDebug => Token::from("debug-key-rotate-colliders"),
		}
	}
}

impl IterFinite for DebugKey {
	fn iterator() -> FiniteIter<Self> {
		FiniteIter(Some(Self::RotateColliderDebug))
	}

	fn next(current: &FiniteIter<Self>) -> Option<Self> {
		match current.0? {
			DebugKey::RotateColliderDebug => None,
		}
	}
}

impl InvalidUserInput for DebugKey {
	fn invalid_input(&self) -> &[UserInput] {
		const { &[] }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_all_keys() {
		assert_eq!(
			vec![DebugKey::RotateColliderDebug],
			DebugKey::iterator().take(100).collect::<Vec<_>>()
		);
	}
}
