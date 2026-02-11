use bevy::prelude::*;
use std::{collections::HashMap, hash::Hash};

#[derive(Component, Debug, PartialEq)]
pub(crate) struct MountPoints<T>(pub(crate) HashMap<T, Entity>)
where
	T: Eq + Hash;

impl<T> Default for MountPoints<T>
where
	T: Eq + Hash,
{
	fn default() -> Self {
		Self(HashMap::default())
	}
}
