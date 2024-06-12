use crate::skills::SkillState;
use common::traits::iteration::{Iter, IterFinite};

impl IterFinite for SkillState {
	fn iterator() -> Iter<Self> {
		Iter(Some(SkillState::Aim))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			SkillState::Aim => Some(SkillState::Active),
			SkillState::Active => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_states() {
		assert_eq!(
			vec![SkillState::Aim, SkillState::Active],
			SkillState::iterator().collect::<Vec<_>>()
		)
	}
}
