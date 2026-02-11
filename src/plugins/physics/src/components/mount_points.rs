use bevy::prelude::*;
use common::{tools::bone_name::BoneName, traits::thread_safe::ThreadSafe};
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

#[derive(Component, Debug, PartialEq, Clone)]
#[require(MountPoints<T>)]
pub struct MountPointsDefinition<T>(pub(crate) HashMap<BoneName, T>)
where
	T: Eq + Hash + ThreadSafe;

impl<T> Default for MountPointsDefinition<T>
where
	T: Eq + Hash + ThreadSafe,
{
	fn default() -> Self {
		Self(HashMap::default())
	}
}
