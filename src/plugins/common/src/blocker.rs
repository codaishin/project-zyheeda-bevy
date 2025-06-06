use crate::traits::iteration::{Iter, IterFinite};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Blocker {
	Physical,
	Force,
	Character,
}

impl Blocker {
	pub fn all() -> Iter<Blocker> {
		Blocker::iterator()
	}
}

impl IterFinite for Blocker {
	fn iterator() -> Iter<Self> {
		Iter(Some(Blocker::Physical))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			Blocker::Physical => Some(Blocker::Force),
			Blocker::Force => Some(Blocker::Character),
			Blocker::Character => None,
		}
	}
}

#[derive(Component, Default, Debug, PartialEq)]
pub struct Blockers(pub HashSet<Blocker>);

impl<const N: usize> From<[Blocker; N]> for Blockers {
	fn from(blockers: [Blocker; N]) -> Self {
		Self(HashSet::from(blockers))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iterate() {
		assert_eq!(
			vec![Blocker::Physical, Blocker::Force, Blocker::Character],
			Blocker::iterator().take(100).collect::<Vec<_>>()
		);
	}
}
