use bevy::{
	ecs::system::{Res, ResMut},
	input::{mouse::MouseButton, ButtonInput},
	state::state::{NextState, State},
};
use common::states::mouse_context::MouseContext;

pub(crate) fn release_triggered_mouse_context(
	mouse_input: Res<ButtonInput<MouseButton>>,
	context: Res<State<MouseContext>>,
	mut next_mouse_context: ResMut<NextState<MouseContext>>,
) {
	if !mouse_input.just_released(MouseButton::Left) {
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
		input::{keyboard::KeyCode, mouse::MouseButton, ButtonInput},
		state::app::{AppExtStates, StatesPlugin},
	};

	fn setup() -> App {
		let mut app = App::new();

		app.add_plugins(StatesPlugin);
		app.init_state::<MouseContext>();
		app.init_resource::<ButtonInput<MouseButton>>();
		app.add_systems(Update, release_triggered_mouse_context);

		app
	}

	#[test]
	fn release_from_triggered() {
		let mut app = setup();

		let mut mouse_buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
		mouse_buttons.press(MouseButton::Left);
		mouse_buttons.release(MouseButton::Left);
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Triggered(KeyCode::KeyU));

		app.update();
		app.update();

		let context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::JustReleased(KeyCode::KeyU), context);
	}

	#[test]
	fn release_from_just_triggered() {
		let mut app = setup();

		let mut mouse_buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
		mouse_buttons.press(MouseButton::Left);
		mouse_buttons.release(MouseButton::Left);
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::JustTriggered(KeyCode::KeyU));

		app.update();
		app.update();

		let context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::JustReleased(KeyCode::KeyU), context);
	}

	#[test]
	fn do_not_release_when_mouse_left_not_just_released() {
		let mut app = setup();

		let mut mouse_buttons = app.world_mut().resource_mut::<ButtonInput<MouseButton>>();
		mouse_buttons.release(MouseButton::Left);
		mouse_buttons.clear_just_released(MouseButton::Left);

		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Triggered(KeyCode::KeyU));

		app.update();
		app.update();

		let context = app
			.world()
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::Triggered(KeyCode::KeyU), context);
	}
}
