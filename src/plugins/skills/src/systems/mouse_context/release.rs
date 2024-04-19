use bevy::{
	ecs::{
		schedule::{NextState, State},
		system::{Res, ResMut},
	},
	input::{mouse::MouseButton, ButtonInput},
};
use common::states::MouseContext;

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
		ecs::schedule::{NextState, State},
		input::{keyboard::KeyCode, mouse::MouseButton, ButtonInput},
	};

	#[test]
	fn release_from_triggered() {
		let mut app = App::new();
		let mut mouse_input = ButtonInput::<MouseButton>::default();

		mouse_input.press(MouseButton::Left);
		app.update();
		mouse_input.release(MouseButton::Left);

		app.insert_resource(mouse_input);
		app.init_state::<MouseContext>();
		app.world
			.get_resource_mut::<NextState<MouseContext>>()
			.unwrap()
			.set(MouseContext::Triggered(KeyCode::KeyU));

		app.add_systems(Update, release_triggered_mouse_context);
		app.update();
		app.update();

		let context = app
			.world
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::JustReleased(KeyCode::KeyU), context);
	}

	#[test]
	fn release_from_just_triggered() {
		let mut app = App::new();
		let mut mouse_input = ButtonInput::<MouseButton>::default();

		mouse_input.press(MouseButton::Left);
		app.update();
		mouse_input.release(MouseButton::Left);

		app.insert_resource(mouse_input);
		app.init_state::<MouseContext>();
		app.world
			.get_resource_mut::<NextState<MouseContext>>()
			.unwrap()
			.set(MouseContext::JustTriggered(KeyCode::KeyU));

		app.add_systems(Update, release_triggered_mouse_context);
		app.update();
		app.update();

		let context = app
			.world
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::JustReleased(KeyCode::KeyU), context);
	}

	#[test]
	fn do_not_release_when_mouse_left_not_just_released() {
		let mut app = App::new();
		let mut mouse_input = ButtonInput::<MouseButton>::default();

		mouse_input.press(MouseButton::Left);
		app.update();
		mouse_input.release(MouseButton::Left);
		app.update();
		mouse_input.clear_just_released(MouseButton::Left);

		app.insert_resource(mouse_input);
		app.init_state::<MouseContext>();
		app.world
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Triggered(KeyCode::KeyU));

		app.add_systems(Update, release_triggered_mouse_context);
		app.update();
		app.update();

		let context = app
			.world
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::Triggered(KeyCode::KeyU), context);
	}
}
