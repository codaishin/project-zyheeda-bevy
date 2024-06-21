use bevy::prelude::{Added, Component, Query};

pub(crate) fn added<TComponent: Component>(query: Query<(), Added<TComponent>>) -> bool {
	!query.is_empty()
}

#[cfg(test)]
mod test {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::system::In,
		prelude::{Commands, IntoSystem, Resource},
	};
	use common::test_tools::utils::SingleThreadedApp;

	#[derive(Resource, Debug, PartialEq)]
	struct _Result(bool);

	#[derive(Component)]
	struct _Component;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);
		app.add_systems(
			Update,
			added::<_Component>.pipe(|added: In<bool>, mut commands: Commands| {
				commands.insert_resource(_Result(added.0));
			}),
		);

		app
	}

	#[test]
	fn return_true_when_added() {
		let mut app = setup();
		app.world.spawn(_Component);

		app.update();

		assert_eq!(&_Result(true), app.world.resource::<_Result>())
	}

	#[test]
	fn return_false_when_empty() {
		let mut app = setup();

		app.update();

		assert_eq!(&_Result(false), app.world.resource::<_Result>())
	}

	#[test]
	fn return_false_when_not_empty_but_none_added() {
		let mut app = setup();
		app.world.spawn(_Component);

		app.update();
		app.update();

		assert_eq!(&_Result(false), app.world.resource::<_Result>())
	}
}
