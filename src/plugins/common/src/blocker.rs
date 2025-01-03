use crate::traits::iteration::{Iter, IterFinite};
use bevy::prelude::*;
use std::collections::HashSet;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Blocker {
	Physical,
	Force,
}

impl Blocker {
	pub fn insert<const N: usize>(blockers: [Blocker; N]) -> BlockerInsertCommand {
		BlockerInsertCommand(HashSet::from(blockers))
	}
}

impl IterFinite for Blocker {
	fn iterator() -> Iter<Self> {
		Iter(Some(Blocker::Physical))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			Blocker::Physical => Some(Blocker::Force),
			Blocker::Force => None,
		}
	}
}

#[derive(Component, Debug, PartialEq)]
pub struct BlockerInsertCommand(pub HashSet<Blocker>);
