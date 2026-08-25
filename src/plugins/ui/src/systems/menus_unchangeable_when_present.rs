use crate::MenusChangeable;
use bevy::prelude::*;

impl<T> MenusUnchangeableWhenPresent for T where T: Component {}

pub(crate) trait MenusUnchangeableWhenPresent: Component + Sized {
	fn menus_unchangeable_when_present(
		current_state: Res<State<MenusChangeable>>,
		mut next_state: ResMut<NextState<MenusChangeable>>,
		blockers: Query<(), With<Self>>,
	) {
		match current_state.get() {
			MenusChangeable(true) if !blockers.is_empty() => next_state.set(MenusChangeable(false)),
			MenusChangeable(false) if blockers.is_empty() => next_state.set(MenusChangeable(true)),
			_ => {}
		};
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::MenusChangeable;
	use bevy::state::app::StatesPlugin;
	use testing::SingleThreadedApp;

	#[derive(Component)]
	struct _Component;

	#[derive(Resource, Debug, PartialEq)]
	struct _StateChanged(bool);

	impl _StateChanged {
		fn update(mut commands: Commands, next_state: ResMut<NextState<MenusChangeable>>) {
			commands.insert_resource(_StateChanged(next_state.is_changed()));
		}
	}

	fn setup(state: MenusChangeable) -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		app.insert_state(state);
		app.add_systems(
			Update,
			(
				_Component::menus_unchangeable_when_present,
				_StateChanged::update,
			)
				.chain(),
		);

		app
	}

	#[test]
	fn set_state_to_unchangeable_when_added() {
		let mut app = setup(MenusChangeable(true));
		app.world_mut().spawn(_Component);

		app.update();
		app.update();

		assert_eq!(
			&MenusChangeable(false),
			app.world().resource::<State<MenusChangeable>>().get()
		);
	}

	#[test]
	fn do_not_set_state_to_unchangeable_when_not_added() {
		#[derive(Component)]
		struct _Other;

		let mut app = setup(MenusChangeable(true));
		app.world_mut().spawn(_Other);

		app.update();
		app.update();

		assert_eq!(
			&MenusChangeable(true),
			app.world().resource::<State<MenusChangeable>>().get()
		);
	}

	#[test]
	fn set_state_to_changeable_when_removed() {
		let mut app = setup(MenusChangeable(true));
		let entity = app.world_mut().spawn(_Component).id();

		app.update();
		app.world_mut().entity_mut(entity).remove::<_Component>();
		app.update();
		app.update();

		assert_eq!(
			&MenusChangeable(true),
			app.world().resource::<State<MenusChangeable>>().get()
		);
	}

	#[test]
	fn do_not_update_state_when_no_change_occurred() {
		let mut app = setup(MenusChangeable(true));

		app.update();
		app.update();

		assert_eq!(
			&_StateChanged(false),
			app.world().resource::<_StateChanged>()
		);
	}
}
