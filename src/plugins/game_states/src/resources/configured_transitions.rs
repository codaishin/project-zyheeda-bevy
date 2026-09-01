use bevy::prelude::*;
use common::prelude::*;
use std::{collections::HashSet, fmt::Debug, hash::Hash};

#[derive(Resource, Debug, PartialEq)]
pub(crate) struct ConfiguredTransitions<T>(pub(crate) HashSet<T>)
where
	T: ThreadSafe + Eq + Hash;

impl<T> Default for ConfiguredTransitions<T>
where
	T: ThreadSafe + Eq + Hash,
{
	fn default() -> Self {
		Self(HashSet::default())
	}
}
