use crate::{traits::accessors::get::TryApplyOn, zyheeda_commands::ZyheedaCommands};
use bevy::{
	ecs::{query::QueryFilter, relationship::Relationship},
	prelude::*,
};

impl<TFilter> LinkToRoot for TFilter where TFilter: QueryFilter {}

pub trait LinkToRoot: QueryFilter + Sized {
	fn link_to_root<TRelationship>(
		mut commands: ZyheedaCommands,
		relationship_targets: Query<(), Without<ChildOf>>,
		candidates: Query<Entity, Self>,
		ancestors: Query<&ChildOf>,
	) where
		TRelationship: Relationship,
	{
		for candidate in candidates {
			let target_ancestor = ancestors
				.iter_ancestors(candidate)
				.find(is_root(relationship_targets));

			let Some(ancestor) = target_ancestor else {
				continue;
			};

			commands.try_apply_on(&candidate, |mut e| {
				e.try_insert(TRelationship::from(ancestor));
			});
		}
	}
}

fn is_root(targets: Query<(), Without<ChildOf>>) -> impl Fn(&Entity) -> bool {
	move |entity| targets.contains(*entity)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::thread_safe::ThreadSafe;
	use bevy::ecs::entity::EntityHashSet;
	use testing::SingleThreadedApp;

	#[derive(Component, Default)]
	#[relationship_target(relationship = _NodeOf)]
	struct _Root(EntityHashSet);

	#[derive(Component, Debug, PartialEq)]
	#[relationship(relationship_target = _Root)]
	struct _NodeOf(Entity);

	fn setup<TFilter>() -> App
	where
		TFilter: QueryFilter + ThreadSafe,
	{
		let mut app = App::new().single_threaded(Update);

		app.add_systems(Update, TFilter::link_to_root::<_NodeOf>);

		app
	}

	#[test]
	fn insert_child_relationship() {
		let mut app = setup::<()>();
		let target = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn(ChildOf(target)).id();

		app.update();

		assert_eq!(
			Some(&_NodeOf(target)),
			app.world().entity(child).get::<_NodeOf>(),
		);
	}

	#[test]
	fn insert_nested_child_relationship() {
		let mut app = setup::<()>();
		let target = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn(ChildOf(target)).id();
		let child_of_child = app.world_mut().spawn(ChildOf(child)).id();

		app.update();

		assert_eq!(
			Some(&_NodeOf(target)),
			app.world().entity(child_of_child).get::<_NodeOf>(),
		);
	}

	#[test]
	fn insert_nested_child_relationship_filtered() {
		#[derive(Component)]
		struct _Filtered;

		let mut app = setup::<Added<_Filtered>>();
		let target = app.world_mut().spawn_empty().id();
		let child = app.world_mut().spawn(ChildOf(target)).id();
		let child_of_child = app.world_mut().spawn((ChildOf(child), _Filtered)).id();

		app.update();

		assert_eq!(
			[None, Some(&_NodeOf(target))],
			app.world()
				.entity([child, child_of_child])
				.map(|c| c.get::<_NodeOf>()),
		);
	}
}
