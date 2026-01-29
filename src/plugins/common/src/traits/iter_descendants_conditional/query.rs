use crate::traits::iter_descendants_conditional::IterDescendantsConditional;
use bevy::{ecs::query::QueryData, prelude::*};
use std::collections::VecDeque;

impl<'w, 's, D> IterDescendantsConditional<'w, 's, D> for Query<'w, 's, D>
where
	D: QueryData,
	D::ReadOnly: QueryData<Item<'w, 's> = &'w Children>,
{
	fn iter_descendants_conditional<TPredicate>(
		&'w self,
		root: Entity,
		predicate: TPredicate,
	) -> super::IterDescendantsWithPredicate<'w, 's, D, TPredicate>
	where
		TPredicate: Fn(Entity) -> bool,
	{
		let mut remaining = VecDeque::new();
		if let Ok(children) = self.get(root) {
			remaining.extend(children);
		}

		super::IterDescendantsWithPredicate {
			children: self,
			remaining,
			predicate,
		}
	}
}

impl<'w, 's, D, TPredicate> Iterator for super::IterDescendantsWithPredicate<'w, 's, D, TPredicate>
where
	TPredicate: Fn(Entity) -> bool,
	D: QueryData,
	D::ReadOnly: QueryData<Item<'w, 's> = &'w Children>,
{
	type Item = Entity;

	fn next(&mut self) -> Option<Self::Item> {
		let next = self.next_matching()?;

		if let Ok(children) = self.children.get(next) {
			self.remaining.extend(children);
		}

		Some(next)
	}
}

impl<'w, 's, D, TPredicate> super::IterDescendantsWithPredicate<'w, 's, D, TPredicate>
where
	TPredicate: Fn(Entity) -> bool,
	D: QueryData,
	D::ReadOnly: QueryData<Item<'w, 's> = &'w Children>,
{
	fn next_matching(&mut self) -> Option<Entity> {
		while let Some(next) = self.remaining.pop_front() {
			if !(self.predicate)(next) {
				continue;
			}

			return Some(next);
		}

		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Skip;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn iter_descendants() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn(ChildOf(entity)).id();

		app.world_mut()
			.run_system_once(move |children: Query<&Children>| {
				let d = children.iter_descendants_conditional(entity, |_| true);

				assert_eq!(vec![child], d.collect::<Vec<_>>());
			})
	}

	#[test]
	fn iter_descendants_deep() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn(ChildOf(entity)).id();
		let deep_child = app.world_mut().spawn(ChildOf(child)).id();

		app.world_mut()
			.run_system_once(move |children: Query<&Children>| {
				let d = children.iter_descendants_conditional(entity, |_| true);

				assert_eq!(vec![child, deep_child], d.collect::<Vec<_>>());
			})
	}

	#[test]
	fn empty_when_sub_tree_skipped() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn((ChildOf(entity), _Skip)).id();
		app.world_mut().spawn(ChildOf(child));

		app.world_mut()
			.run_system_once(move |children: Query<&Children>, skips: Query<&_Skip>| {
				let mut d = children.iter_descendants_conditional(entity, |e| !skips.contains(e));

				assert!(d.next().is_none());
			})
	}

	#[test]
	fn iter_children_ignoring_skipped_subtrees() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app.world_mut().spawn_empty().id();
		let child_a = app.world_mut().spawn(ChildOf(entity)).id();
		app.world_mut().spawn((ChildOf(entity), _Skip));
		let child_b = app.world_mut().spawn(ChildOf(entity)).id();

		app.world_mut()
			.run_system_once(move |children: Query<&Children>, skips: Query<&_Skip>| {
				let d = children.iter_descendants_conditional(entity, |e| !skips.contains(e));

				assert_eq!(vec![child_a, child_b], d.collect::<Vec<_>>());
			})
	}
}
