use crate::skills::SkillState;
use common::traits::iteration::{Iter, IterKey};

impl IterKey for SkillState {
	fn iterator() -> Iter<Self> {
		Iter(Some(SkillState::Aim))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			SkillState::Aim => Some(SkillState::PreCast),
			SkillState::PreCast => Some(SkillState::Active),
			SkillState::Active => Some(SkillState::AfterCast),
			SkillState::AfterCast => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_states() {
		assert_eq!(
			vec![
				SkillState::Aim,
				SkillState::PreCast,
				SkillState::Active,
				SkillState::AfterCast
			],
			SkillState::iterator().collect::<Vec<_>>()
		)
	}
}
