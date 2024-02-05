use crate::states::MouseContext;
use bevy::{
	ecs::{
		schedule::{NextState, State},
		system::{Res, ResMut},
	},
	input::{mouse::MouseButton, Input},
};

pub fn release_triggered_mouse_context(
	mouse_input: Res<Input<MouseButton>>,
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
	use crate::states::MouseContext;
	use bevy::{
		app::{App, Update},
		ecs::schedule::{NextState, State},
		input::{keyboard::KeyCode, mouse::MouseButton, Input},
	};

	#[test]
	fn release_from_triggered() {
		let mut app = App::new();
		let mut mouse_input = Input::<MouseButton>::default();

		mouse_input.press(MouseButton::Left);
		app.update();
		mouse_input.release(MouseButton::Left);

		app.insert_resource(mouse_input);
		app.add_state::<MouseContext>();
		app.world
			.get_resource_mut::<NextState<MouseContext>>()
			.unwrap()
			.set(MouseContext::Triggered(KeyCode::U));

		app.add_systems(Update, release_triggered_mouse_context);
		app.update();
		app.update();

		let context = app
			.world
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::JustReleased(KeyCode::U), context);
	}

	#[test]
	fn release_from_just_triggered() {
		let mut app = App::new();
		let mut mouse_input = Input::<MouseButton>::default();

		mouse_input.press(MouseButton::Left);
		app.update();
		mouse_input.release(MouseButton::Left);

		app.insert_resource(mouse_input);
		app.add_state::<MouseContext>();
		app.world
			.get_resource_mut::<NextState<MouseContext>>()
			.unwrap()
			.set(MouseContext::JustTriggered(KeyCode::U));

		app.add_systems(Update, release_triggered_mouse_context);
		app.update();
		app.update();

		let context = app
			.world
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::JustReleased(KeyCode::U), context);
	}

	#[test]
	fn do_not_release_when_mouse_left_not_just_released() {
		let mut app = App::new();
		let mut mouse_input = Input::<MouseButton>::default();

		mouse_input.press(MouseButton::Left);
		app.update();
		mouse_input.release(MouseButton::Left);
		app.update();
		mouse_input.clear_just_released(MouseButton::Left);

		app.insert_resource(mouse_input);
		app.add_state::<MouseContext>();
		app.world
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Triggered(KeyCode::U));

		app.add_systems(Update, release_triggered_mouse_context);
		app.update();
		app.update();

		let context = app
			.world
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::Triggered(KeyCode::U), context);
	}
}
