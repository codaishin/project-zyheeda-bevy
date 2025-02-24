use crate::traits::{thread_safe::ThreadSafe, try_insert_on::TryInsertOn};
use bevy::{ecs::query::QueryFilter, prelude::*};
use std::marker::PhantomData;

pub struct InsertOn<TComponent, TFilter1 = (), TFilter2 = Added<TComponent>>(
	PhantomData<(TComponent, TFilter1, TFilter2)>,
)
where
	TComponent: Component,
	(TFilter1, TFilter2): QueryFilter;

pub trait InsertRequired<TComponent, TFilter1, TFilter2>
where
	TComponent: Component,
	(TFilter1, TFilter2): QueryFilter,
{
	#[allow(clippy::type_complexity)]
	fn required<TRequired>(
		constructor: impl Fn(&TComponent) -> TRequired + ThreadSafe,
	) -> impl Fn(Commands, Query<(Entity, &TComponent), (TFilter1, TFilter2)>)
	where
		TRequired: Component;
}

impl<TComponent, TFilter1, TFilter2> InsertRequired<TComponent, TFilter1, TFilter2>
	for InsertOn<TComponent, TFilter1, TFilter2>
where
	TComponent: Component,
	(TFilter1, TFilter2): QueryFilter,
{
	fn required<TRequired>(
		constructor: impl Fn(&TComponent) -> TRequired + ThreadSafe,
	) -> impl Fn(Commands, Query<(Entity, &TComponent), (TFilter1, TFilter2)>)
	where
		TRequired: Component,
	{
		move |mut commands: Commands,
		      entities: Query<(Entity, &TComponent), (TFilter1, TFilter2)>| {
			for (entity, component) in &entities {
				let required = constructor(component);
				commands.try_insert_on(entity, required);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::traits::thread_safe::ThreadSafe;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Component;

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Required(&'static str);

	fn setup<TFilter1, TFilter2>(constructor: impl Fn(&_Component) -> _Required + ThreadSafe) -> App
	where
		(TFilter1, TFilter2): QueryFilter + 'static,
	{
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			InsertOn::<_Component, TFilter1, TFilter2>::required::<_Required>(constructor),
		);

		app
	}

	#[test]
	fn add_associated_component() {
		let mut app = setup::<(), ()>(|_| _Required("overridden"));
		let entity = app.world_mut().spawn(_Component).id();

		app.update();

		assert_eq!(
			Some(&_Required("overridden")),
			app.world().entity(entity).get::<_Required>(),
		)
	}

	#[test]
	fn do_not_add_associated_component_when_no_agent() {
		let mut app = setup::<(), ()>(|_| _Required::default());
		let entity = app.world_mut().spawn_empty().id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Required>(),)
	}

	#[test]
	fn do_not_add_associated_component_when_filter1_constraint_violated() {
		#[derive(Component)]
		struct _Other;

		let mut app = setup::<With<_Other>, ()>(|_| _Required::default());
		let entity = app.world_mut().spawn(_Component).id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Required>(),)
	}

	#[test]
	fn do_not_add_associated_component_when_filter2_constraint_violated() {
		#[derive(Component)]
		struct _Other;

		let mut app = setup::<(), With<_Other>>(|_| _Required::default());
		let entity = app.world_mut().spawn(_Component).id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Required>(),)
	}
}
