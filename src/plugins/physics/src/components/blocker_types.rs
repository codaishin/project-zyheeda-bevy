use bevy::prelude::*;
use common::traits::handles_physics::physical_bodies::Blocker;
use std::collections::HashSet;

#[derive(Component, Debug, PartialEq, Default)]
pub(crate) struct BlockerTypes(pub(crate) HashSet<Blocker>);

impl<TBlocks> From<TBlocks> for BlockerTypes
where
	TBlocks: IntoIterator<Item = Blocker>,
{
	fn from(blocks: TBlocks) -> Self {
		Self(HashSet::from_iter(blocks))
	}
}
