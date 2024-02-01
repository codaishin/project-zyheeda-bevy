use super::{Iter, IterKey};
use crate::skill::SkillState;

impl IterKey for SkillState {
	fn iterator() -> Iter<Self> {
		Iter(Some(SkillState::PreCast))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
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
				SkillState::PreCast,
				SkillState::Active,
				SkillState::AfterCast
			],
			SkillState::iterator().collect::<Vec<_>>()
		)
	}
}
