use crate::traits::try_insert_on::TryInsertOn;
use bevy::prelude::*;

impl<TComponent> InsertAssociated for TComponent where Self: Component {}

pub trait InsertAssociated
where
	Self: Component + Sized,
{
	fn insert_associated<TBundle>(
		configure: Configure<TBundle>,
	) -> impl Fn(Commands, Query<Entity, Added<Self>>)
	where
		TBundle: Bundle + Default,
	{
		let configure = match configure {
			Configure::Apply(configure) => configure,
			_ => |_: &mut TBundle| {},
		};

		move |mut commands: Commands, entities: Query<Entity, Added<Self>>| {
			for entity in &entities {
				let mut bundle = TBundle::default();
				configure(&mut bundle);
				commands.try_insert_on(entity, bundle);
			}
		}
	}
}

pub enum Configure<TBundle> {
	LeaveAsIs,
	Apply(fn(&mut TBundle)),
}

#[cfg(test)]
mod tests {
	use super::*;
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

	#[derive(Component, Debug, PartialEq, Default)]
	struct _Associated(&'static str);

	fn setup(configure: Configure<_Associated>) -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(Update, _Agent::insert_associated::<_Associated>(configure));

		app
	}

	#[test]
	fn add_associated_component() {
		let mut app = setup(Configure::Apply(|bundle: &mut _Associated| {
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
		let mut app = setup(Configure::LeaveAsIs);
		let entity = app.world_mut().spawn_empty().id();

		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Associated>(),)
	}

	#[test]
	fn do_not_add_associated_component_when_agent_not_new() {
		let mut app = setup(Configure::LeaveAsIs);
		let entity = app.world_mut().spawn(_Agent).id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<_Associated>();
		app.update();

		assert_eq!(None, app.world().entity(entity).get::<_Associated>(),)
	}
}
