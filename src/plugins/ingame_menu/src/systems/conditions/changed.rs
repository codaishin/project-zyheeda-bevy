use bevy::prelude::{Changed, Component, Query, With};

pub(crate) fn changed<TAgent: Component, TComponent: Component>(
	query: Query<(), (With<TAgent>, Changed<TComponent>)>,
) -> bool {
	!query.is_empty()
}

#[cfg(test)]
mod tests {
	use std::ops::DerefMut;

	use super::*;
	use bevy::{
		app::{App, Update},
		prelude::{Commands, In, IntoSystem, Resource},
	};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Component)]
	struct _Agent;

	#[derive(Component)]
	struct _Component;

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(bool);

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			changed::<_Agent, _Component>.pipe(|result: In<bool>, mut commands: Commands| {
				commands.insert_resource(_Result(result.0));
			}),
		);

		app
	}

	#[test]
	fn true_when_added() {
		let mut app = setup();

		app.world_mut().spawn((_Agent, _Component));
		app.update();

		assert_eq!(&_Result(true), app.world().resource::<_Result>());
	}

	#[test]
	fn false_when_non_added() {
		let mut app = setup();

		app.update();

		assert_eq!(&_Result(false), app.world().resource::<_Result>());
	}

	#[test]
	fn false_when_added_long_ago() {
		let mut app = setup();

		app.world_mut().spawn((_Agent, _Component));
		app.update();
		app.update();

		assert_eq!(&_Result(false), app.world().resource::<_Result>());
	}

	#[test]
	fn true_when_mutably_dereferenced() {
		let mut app = setup();

		let agent = app.world_mut().spawn((_Agent, _Component)).id();
		app.update();
		app.world_mut()
			.entity_mut(agent)
			.get_mut::<_Component>()
			.unwrap()
			.deref_mut();
		app.update();

		assert_eq!(&_Result(true), app.world().resource::<_Result>());
	}
}
