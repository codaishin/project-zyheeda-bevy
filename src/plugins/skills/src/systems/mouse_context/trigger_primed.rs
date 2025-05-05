use bevy::prelude::*;
use common::{states::mouse_context::MouseContext, tools::keys::user_input::UserInput};

pub(crate) fn trigger_primed_mouse_context(
	mouse_input: Res<ButtonInput<UserInput>>,
	mouse_context: Res<State<MouseContext>>,
	mut next_mouse_context: ResMut<NextState<MouseContext>>,
) {
	if !mouse_input.just_pressed(UserInput::from(MouseButton::Left)) {
		return;
	}
	let MouseContext::Primed(key) = mouse_context.get() else {
		return;
	};
	next_mouse_context.set(MouseContext::JustTriggered(*key));
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		input::{ButtonInput, keyboard::KeyCode, mouse::MouseButton},
		state::app::{AppExtStates, StatesPlugin},
	};
	use common::tools::keys::user_input::UserInput;

	fn setup() -> App {
		let mut app = App::new();

		app.add_plugins(StatesPlugin);
		app.init_state::<MouseContext>();
		app.init_resource::<ButtonInput<UserInput>>();
		app.add_systems(Update, trigger_primed_mouse_context);

		app
	}

	#[test]
	fn trigger() {
		let mut app = setup();

		app.world_mut()
			.resource_mut::<ButtonInput<UserInput>>()
			.press(UserInput::from(MouseButton::Left));
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Primed(UserInput::from(KeyCode::KeyU)));

		app.update();
		app.update();

		let context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(
			&MouseContext::JustTriggered(UserInput::from(KeyCode::KeyU)),
			context
		);
	}

	#[test]
	fn do_not_trigger_when_mouse_left_not_just_pressed() {
		let mut app = setup();

		let mut mouse_buttons = app.world_mut().resource_mut::<ButtonInput<UserInput>>();
		mouse_buttons.press(UserInput::from(MouseButton::Left));
		mouse_buttons.clear_just_pressed(UserInput::from(MouseButton::Left));
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Primed(UserInput::from(KeyCode::KeyU)));

		app.update();
		app.update();

		let context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(
			&MouseContext::Primed(UserInput::from(KeyCode::KeyU)),
			context
		);
	}

	#[test]
	fn do_not_trigger_when_no_mouse_context_key() {
		let mut app = setup();

		app.world_mut()
			.resource_mut::<ButtonInput<UserInput>>()
			.press(UserInput::from(MouseButton::Left));
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::UI);

		app.update();
		app.update();

		let context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::UI, context);
	}

	#[test]
	fn trigger_other_key() {
		let mut app = setup();

		app.world_mut()
			.resource_mut::<ButtonInput<UserInput>>()
			.press(UserInput::from(MouseButton::Left));
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Primed(UserInput::from(KeyCode::KeyO)));

		app.update();
		app.update();

		let context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(
			&MouseContext::JustTriggered(UserInput::from(KeyCode::KeyO)),
			context
		);
	}
}
