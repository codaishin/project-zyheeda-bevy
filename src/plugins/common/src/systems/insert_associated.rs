use crate::traits::try_insert_on::TryInsertOn;
use bevy::{ecs::query::QueryFilter, prelude::*};
use std::marker::PhantomData;

pub struct InsertOn<TComponent, TFilter1 = (), TFilter2 = Added<TComponent>>(
	PhantomData<(TComponent, TFilter1, TFilter2)>,
)
where
	TComponent: Component,
	(TFilter1, TFilter2): QueryFilter;

pub enum Configure<TComponent, TBundle> {
	LeaveAsIs,
	Apply(fn(&TComponent, &mut TBundle)),
}

pub trait InsertAssociated<TComponent, TFilter1, TFilter2>
where
	TComponent: Component,
	(TFilter1, TFilter2): QueryFilter,
{
	#[allow(clippy::type_complexity)]
	fn associated<TBundle>(
		configure: Configure<TComponent, TBundle>,
	) -> impl Fn(Commands, Query<(Entity, &TComponent), (TFilter1, TFilter2)>)
	where
		TBundle: Bundle + Default;
}

impl<TComponent, TFilter1, TFilter2> InsertAssociated<TComponent, TFilter1, TFilter2>
	for InsertOn<TComponent, TFilter1, TFilter2>
where
	TComponent: Component,
	(TFilter1, TFilter2): QueryFilter,
{
	fn associated<TBundle>(
		configure: Configure<TComponent, TBundle>,
	) -> impl Fn(Commands, Query<(Entity, &TComponent), (TFilter1, TFilter2)>)
	where
		TBundle: Bundle + Default,
	{
		let configure = match configure {
			Configure::Apply(configure) => configure,
			_ => |_: &TComponent, _: &mut TBundle| {},
		};

		move |mut commands: Commands,
		      entities: Query<(Entity, &TComponent), (TFilter1, TFilter2)>| {
			for (entity, component) in &entities {
				let mut bundle = TBundle::default();
				configure(component, &mut bundle);
				commands.try_insert_on(entity, bundle);
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Associated(&'static str);

	fn setup<TFilter1, TFilter2>(configure: Configure<_Agent, _Associated>) -> App
	where
		(TFilter1, TFilter2): QueryFilter + 'static,
	{
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			InsertOn::<_Agent, TFilter1, TFilter2>::associated::<_Associated>(configure),
		);

		app
	}

	#[test]
	fn add_associated_component() {
		let mut app = setup::<(), ()>(Configure::Apply(|&_, bundle: &mut _Associated| {
			*bundle = _Associated("overridden");
		}));
		let entity = app.world_mut().spawn(_Agent).id();

		app.update();

		assert_eq!(
			Some(&_Associated("overridden")),
			app.world().entity(entity).get::<_Associated>(),
		)
	}

	#[test]
	fn do_not_add_associated_component_when_no_agent() {
		let mut app = setup::<(), ()>(Configure::LeaveAsIs);
		let entity = app.world_mut().spawn_empty().id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Associated>(),)
	}

	#[test]
	fn do_not_add_associated_component_when_filter1_constraint_violated() {
		#[derive(Component)]
		struct _Other;

		let mut app = setup::<With<_Other>, ()>(Configure::LeaveAsIs);
		let entity = app.world_mut().spawn(_Agent).id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Associated>(),)
	}

	#[test]
	fn do_not_add_associated_component_when_filter2_constraint_violated() {
		#[derive(Component)]
		struct _Other;

		let mut app = setup::<(), With<_Other>>(Configure::LeaveAsIs);
		let entity = app.world_mut().spawn(_Agent).id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Associated>(),)
	}
}
