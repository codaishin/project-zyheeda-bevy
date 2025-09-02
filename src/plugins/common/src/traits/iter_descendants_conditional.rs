mod query;

use bevy::{ecs::query::QueryData, prelude::*};
use std::collections::VecDeque;

pub trait IterDescendantsConditional<'w, 's, D>
where
	D: QueryData,
	D::ReadOnly: QueryData<Item<'w> = &'w Children>,
{
	/// Iterate descendants
	///
	/// Skip any sub-hierarchies whose root entities do not match the `predicate`
	fn iter_descendants_conditional<TPredicate>(
		&'w self,
		root: Entity,
		predicate: TPredicate,
	) -> IterDescendantsWithPredicate<'w, 's, D, TPredicate>
	where
		TPredicate: Fn(Entity) -> bool;
}

pub struct IterDescendantsWithPredicate<'w, 's, D, TPredicate>
where
	D: QueryData,
	D::ReadOnly: QueryData<Item<'w> = &'w Children>,
{
	children: &'w Query<'w, 's, D>,
	remaining: VecDeque<Entity>,
	predicate: TPredicate,
}
