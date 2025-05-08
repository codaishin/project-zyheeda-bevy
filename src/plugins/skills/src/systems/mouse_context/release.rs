use bevy::prelude::*;
use common::{states::mouse_context::MouseContext, tools::action_key::user_input::UserInput};

pub(crate) fn release_triggered_mouse_context(
	mouse_input: Res<ButtonInput<UserInput>>,
	context: Res<State<MouseContext>>,
	mut next_mouse_context: ResMut<NextState<MouseContext>>,
) {
	if !mouse_input.just_released(UserInput::from(MouseButton::Left)) {
		return;
	}
	if let MouseContext::Triggered(key) | MouseContext::JustTriggered(key) = context.get() {
		next_mouse_context.set(MouseContext::JustReleased(*key));
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		input::{ButtonInput, keyboard::KeyCode, mouse::MouseButton},
		state::app::{AppExtStates, StatesPlugin},
	};
	use common::tools::action_key::user_input::UserInput;

	fn setup() -> App {
		let mut app = App::new();

		app.add_plugins(StatesPlugin);
		app.init_state::<MouseContext>();
		app.init_resource::<ButtonInput<UserInput>>();
		app.add_systems(Update, release_triggered_mouse_context);

		app
	}

	#[test]
	fn release_from_triggered() {
		let mut app = setup();

		let mut mouse_buttons = app.world_mut().resource_mut::<ButtonInput<UserInput>>();
		mouse_buttons.press(UserInput::from(MouseButton::Left));
		mouse_buttons.release(UserInput::from(MouseButton::Left));
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Triggered(UserInput::from(KeyCode::KeyU)));

		app.update();
		app.update();

		let context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(
			&MouseContext::JustReleased(UserInput::from(KeyCode::KeyU)),
			context
		);
	}

	#[test]
	fn release_from_just_triggered() {
		let mut app = setup();

		let mut mouse_buttons = app.world_mut().resource_mut::<ButtonInput<UserInput>>();
		mouse_buttons.press(UserInput::from(MouseButton::Left));
		mouse_buttons.release(UserInput::from(MouseButton::Left));
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::JustTriggered(UserInput::from(KeyCode::KeyU)));

		app.update();
		app.update();

		let context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(
			&MouseContext::JustReleased(UserInput::from(KeyCode::KeyU)),
			context
		);
	}

	#[test]
	fn do_not_release_when_mouse_left_not_just_released() {
		let mut app = setup();

		let mut mouse_buttons = app.world_mut().resource_mut::<ButtonInput<UserInput>>();
		mouse_buttons.release(UserInput::from(MouseButton::Left));
		mouse_buttons.clear_just_released(UserInput::from(MouseButton::Left));

		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Triggered(UserInput::from(KeyCode::KeyU)));

		app.update();
		app.update();

		let context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(
			&MouseContext::Triggered(UserInput::from(KeyCode::KeyU)),
			context
		);
	}
}
