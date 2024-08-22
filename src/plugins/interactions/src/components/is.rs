use std::collections::HashSet;

use super::blocker::Blocker;
use crate::traits::Blockable;
use bevy::prelude::Component;

#[derive(Component)]
pub struct Is<T>(pub(crate) T);

#[allow(private_bounds)]
impl<T> Is<T>
where
	T: Blockable,
	Is<T>: Component,
{
	pub fn interacting_with<const N: usize>(blockers: [Blocker; N]) -> Self {
		Is(T::blockable(blockers))
	}
}

pub struct Fragile(pub(crate) HashSet<Blocker>);

impl Blockable for Fragile {
	fn blockable<const N: usize>(blockers: [Blocker; N]) -> Self {
		Fragile(HashSet::from(blockers))
	}
}

pub struct InterruptableRay(pub(crate) HashSet<Blocker>);

impl Blockable for InterruptableRay {
	fn blockable<const N: usize>(blockers: [Blocker; N]) -> Self {
		InterruptableRay(HashSet::from(blockers))
	}
}
