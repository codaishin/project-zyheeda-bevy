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
	pub fn interacting_with<TBlockers>(blockers: TBlockers) -> Self
	where
		TBlockers: IntoIterator<Item = Blocker>,
	{
		Is(T::blockable(blockers))
	}
}

pub struct Fragile(pub(crate) HashSet<Blocker>);

impl Blockable for Fragile {
	fn blockable<TBlockers>(blockers: TBlockers) -> Self
	where
		TBlockers: IntoIterator<Item = Blocker>,
	{
		Fragile(HashSet::from_iter(blockers))
	}
}

pub struct InterruptableRay(pub(crate) HashSet<Blocker>);

impl Blockable for InterruptableRay {
	fn blockable<TBlockers>(blockers: TBlockers) -> Self
	where
		TBlockers: IntoIterator<Item = Blocker>,
	{
		InterruptableRay(HashSet::from_iter(blockers))
	}
}
