use bevy::prelude::*;
use common::traits::thread_safe::ThreadSafe;
use std::collections::HashSet;

#[derive(Resource, Default)]
pub(crate) struct GameStatesContext<T>
where
	T: ThreadSafe,
{
	pub(crate) states: HashSet<T>,
}
