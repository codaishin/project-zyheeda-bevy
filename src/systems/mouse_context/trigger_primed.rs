use bevy::{
	ecs::{
		schedule::{NextState, State},
		system::{Res, ResMut},
	},
	input::{mouse::MouseButton, Input},
};
use common::states::MouseContext;

pub fn trigger_primed_mouse_context(
	mouse_input: Res<Input<MouseButton>>,
	mouse_context: Res<State<MouseContext>>,
	mut next_mouse_context: ResMut<NextState<MouseContext>>,
) {
	if !mouse_input.just_pressed(MouseButton::Left) {
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
		ecs::schedule::NextState,
		input::{keyboard::KeyCode, mouse::MouseButton, Input},
	};

	#[test]
	fn trigger() {
		let mut app = App::new();
		let mut mouse_input = Input::<MouseButton>::default();

		mouse_input.press(MouseButton::Left);
		app.insert_resource(mouse_input);
		app.add_state::<MouseContext>();
		app.world
			.get_resource_mut::<NextState<MouseContext>>()
			.unwrap()
			.set(MouseContext::Primed(KeyCode::U));

		app.add_systems(Update, trigger_primed_mouse_context);
		app.update();
		app.update();

		let context = app
			.world
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::JustTriggered(KeyCode::U), context);
	}

	#[test]
	fn do_not_trigger_when_mouse_left_not_just_pressed() {
		let mut app = App::new();
		let mut mouse_input = Input::<MouseButton>::default();

		mouse_input.press(MouseButton::Left);
		app.insert_resource(mouse_input);
		app.add_state::<MouseContext>();
		app.world
			.get_resource_mut::<NextState<MouseContext>>()
			.unwrap()
			.set(MouseContext::Primed(KeyCode::U));

		app.update();
		app.world
			.get_resource_mut::<Input<MouseButton>>()
			.unwrap()
			.clear_just_pressed(MouseButton::Left);
		app.add_systems(Update, trigger_primed_mouse_context);
		app.update();
		app.update();

		let context = app
			.world
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::Primed(KeyCode::U), context);
	}

	#[test]
	fn do_not_trigger_when_no_mouse_context_key() {
		let mut app = App::new();
		let mut mouse_input = Input::<MouseButton>::default();

		mouse_input.press(MouseButton::Left);
		app.insert_resource(mouse_input);
		app.add_state::<MouseContext>();
		app.world
			.get_resource_mut::<NextState<MouseContext>>()
			.unwrap()
			.set(MouseContext::UI);

		app.add_systems(Update, trigger_primed_mouse_context);
		app.update();
		app.update();

		let context = app
			.world
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::UI, context);
	}

	#[test]
	fn trigger_other_key() {
		let mut app = App::new();
		let mut mouse_input = Input::<MouseButton>::default();

		mouse_input.press(MouseButton::Left);
		app.insert_resource(mouse_input);
		app.add_state::<MouseContext>();
		app.world
			.get_resource_mut::<NextState<MouseContext>>()
			.unwrap()
			.set(MouseContext::Primed(KeyCode::O));

		app.add_systems(Update, trigger_primed_mouse_context);
		app.update();
		app.update();

		let context = app
			.world
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::JustTriggered(KeyCode::O), context);
	}
}
