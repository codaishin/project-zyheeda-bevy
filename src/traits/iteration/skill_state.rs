use super::{Iter, IterKey};
use crate::skill::SkillState;

impl IterKey for SkillState {
	fn iterator() -> Iter<Self> {
		Iter(Some(SkillState::PreTransition))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			SkillState::PreTransition => Some(SkillState::PreCast),
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
				SkillState::PreTransition,
				SkillState::PreCast,
				SkillState::Active,
				SkillState::AfterCast
			],
			SkillState::iterator().collect::<Vec<_>>()
		)
	}
}
