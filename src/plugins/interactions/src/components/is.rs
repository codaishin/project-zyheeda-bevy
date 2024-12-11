use crate::traits::Blockable;
use bevy::prelude::Component;
use common::blocker::Blocker;
use std::collections::HashSet;

#[derive(Component)]
pub(crate) struct Is<T>(pub(crate) T);

impl<T> Is<T>
where
	T: Blockable,
	Is<T>: Component,
{
	pub fn interacting_with(blockers: &[Blocker]) -> Self {
		Is(T::blockable(blockers))
	}
}

pub struct Fragile(pub(crate) HashSet<Blocker>);

impl Blockable for Fragile {
	fn blockable(blockers: &[Blocker]) -> Self {
		Fragile(HashSet::from_iter(blockers.iter().cloned()))
	}
}

pub struct InterruptableRay(pub(crate) HashSet<Blocker>);

impl Blockable for InterruptableRay {
	fn blockable(blockers: &[Blocker]) -> Self {
		InterruptableRay(HashSet::from_iter(blockers.iter().cloned()))
	}
}
