use bevy::prelude::*;
use common::components::is_blocker::Blocker;
use std::collections::HashSet;

#[derive(Component, Debug, PartialEq)]
pub(crate) struct BlockedBy(pub(crate) HashSet<Blocker>);

impl<T> From<T> for BlockedBy
where
	T: IntoIterator<Item = Blocker>,
{
	fn from(blockers: T) -> Self {
		Self(HashSet::from_iter(blockers))
	}
}
