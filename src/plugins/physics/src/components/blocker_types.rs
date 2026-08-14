use bevy::prelude::*;
use common::prelude::*;
use std::collections::HashSet;

#[derive(Component, Debug, PartialEq, Default, Clone)]
pub(crate) struct BlockerTypes(pub(crate) HashSet<Blocker>);

impl<TBlocks> From<TBlocks> for BlockerTypes
where
	TBlocks: IntoIterator<Item = Blocker>,
{
	fn from(blocks: TBlocks) -> Self {
		Self(HashSet::from_iter(blocks))
	}
}
