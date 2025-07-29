use bevy::prelude::*;
use common::{
	components::is_blocker::Blocker,
	traits::handles_interactions::{BlockableDefinition, BlockableType},
};
use std::collections::HashSet;

#[derive(Component, Debug, PartialEq)]
pub struct Blockable {
	pub(crate) blockable_type: BlockableType,
	pub(crate) blockers: HashSet<Blocker>,
}

impl BlockableDefinition for Blockable {
	fn new<T>(blockable_type: BlockableType, blocked_by: T) -> Self
	where
		T: IntoIterator<Item = Blocker>,
	{
		Self {
			blockable_type,
			blockers: HashSet::from_iter(blocked_by),
		}
	}
}
