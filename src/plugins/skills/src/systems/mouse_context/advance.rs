use bevy::{
	ecs::system::{Res, ResMut},
	state::state::{NextState, State},
};
use common::states::mouse_context::MouseContext;

pub(crate) fn advance_just_triggered_mouse_context(
	mouse_context: Res<State<MouseContext>>,
	mut next_mouse_context: ResMut<NextState<MouseContext>>,
) {
	let MouseContext::JustTriggered(key) = mouse_context.get() else {
		return;
	};
	next_mouse_context.set(MouseContext::Triggered(*key));
}

pub(crate) fn advance_just_released_mouse_context(
	mouse_context: Res<State<MouseContext>>,
	mut next_mouse_context: ResMut<NextState<MouseContext>>,
) {
	let MouseContext::JustReleased(_) = mouse_context.get() else {
		return;
	};
	next_mouse_context.set(MouseContext::Default);
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		input::keyboard::KeyCode,
		state::app::{AppExtStates, StatesPlugin},
	};
	use common::tools::action_key::user_input::UserInput;

	fn setup() -> App {
		let mut app = App::new();

		app.add_plugins(StatesPlugin);
		app.init_state::<MouseContext>();
		app.add_systems(Update, advance_just_triggered_mouse_context);
		app.add_systems(Update, advance_just_released_mouse_context);

		app
	}

	#[test]
	fn advance_to_triggered() {
		let mut app = setup();

		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::JustTriggered(UserInput::from(KeyCode::KeyB)));

		app.update();
		app.update();

		let mouse_context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(
			&MouseContext::Triggered(UserInput::from(KeyCode::KeyB)),
			mouse_context
		);
	}

	#[test]
	fn do_not_advance_to_triggered_when_no_key_pressed() {
		let mut app = setup();

		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Primed(UserInput::from(KeyCode::KeyB)));

		app.update();
		app.update();

		let mouse_context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(
			&MouseContext::Primed(UserInput::from(KeyCode::KeyB)),
			mouse_context
		);
	}

	#[test]
	fn advance_to_default() {
		let mut app = setup();

		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::JustReleased(UserInput::from(KeyCode::KeyB)));

		app.update();
		app.update();

		let mouse_context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::Default, mouse_context);
	}

	#[test]
	fn do_not_advance_to_default_when_no_key_released() {
		let mut app = setup();

		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Primed(UserInput::from(KeyCode::KeyB)));

		app.update();
		app.update();

		let mouse_context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(
			&MouseContext::Primed(UserInput::from(KeyCode::KeyB)),
			mouse_context
		);
	}
}
