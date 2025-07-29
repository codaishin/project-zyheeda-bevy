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
	pub fn all<TBlockers>() -> TBlockers
	where
		TBlockers: FromIterator<Blocker>,
	{
		Blocker::iterator().collect()
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

// FIXME: Move to interactions plugin, once dependencies between player, enemy and interactions
// is figured out: interactions should depend on player/enemy plugins
#[derive(Component, Default, Debug, PartialEq)]
pub struct IsBlocker(pub HashSet<Blocker>);

impl<T> From<T> for IsBlocker
where
	T: IntoIterator<Item = Blocker>,
{
	fn from(blockers: T) -> Self {
		Self(HashSet::from_iter(blockers))
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
