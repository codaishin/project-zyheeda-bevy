use crate::skills::SkillState;
use common::prelude::*;

impl IterFinite for SkillState {
	fn iterator() -> FiniteIter<Self> {
		FiniteIter(Some(SkillState::Aim))
	}

	fn next(current: &FiniteIter<Self>) -> Option<Self> {
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
