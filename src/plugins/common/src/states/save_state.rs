use crate::traits::iteration::{Iter, IterFinite};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Serialize, Deserialize)]
pub enum SaveState {
	Save,
	AttemptLoad,
	Load,
}

impl IterFinite for SaveState {
	fn iterator() -> Iter<Self> {
		Iter(Some(Self::Save))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			Self::Save => Some(Self::AttemptLoad),
			Self::AttemptLoad => Some(Self::Load),
			Self::Load => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn keys() {
		assert_eq!(
			vec![SaveState::Save, SaveState::AttemptLoad, SaveState::Load],
			SaveState::iterator().collect::<Vec<_>>(),
		);
	}
}
