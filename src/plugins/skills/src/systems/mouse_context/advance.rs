use bevy::ecs::{
	schedule::{NextState, State},
	system::{Res, ResMut},
};
use common::states::MouseContext;

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
		ecs::schedule::{NextState, State},
		input::keyboard::KeyCode,
	};

	#[test]
	fn advance_to_triggered() {
		let mut app = App::new();

		app.init_state::<MouseContext>();
		app.world
			.get_resource_mut::<NextState<MouseContext>>()
			.unwrap()
			.set(MouseContext::JustTriggered(KeyCode::KeyB));

		app.add_systems(Update, advance_just_triggered_mouse_context);
		app.update();
		app.update();

		let mouse_context = app
			.world
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::Triggered(KeyCode::KeyB), mouse_context);
	}

	#[test]
	fn do_not_advance_to_triggered_when_no_key_pressed() {
		let mut app = App::new();

		app.init_state::<MouseContext>();
		app.world
			.get_resource_mut::<NextState<MouseContext>>()
			.unwrap()
			.set(MouseContext::Primed(KeyCode::KeyB));

		app.add_systems(Update, advance_just_triggered_mouse_context);
		app.update();
		app.update();

		let mouse_context = app
			.world
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::Primed(KeyCode::KeyB), mouse_context);
	}

	#[test]
	fn advance_to_default() {
		let mut app = App::new();

		app.init_state::<MouseContext>();
		app.world
			.get_resource_mut::<NextState<MouseContext>>()
			.unwrap()
			.set(MouseContext::JustReleased(KeyCode::KeyB));

		app.add_systems(Update, advance_just_released_mouse_context);
		app.update();
		app.update();

		let mouse_context = app
			.world
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::Default, mouse_context);
	}

	#[test]
	fn do_not_advance_to_default_when_no_key_released() {
		let mut app = App::new();

		app.init_state::<MouseContext>();
		app.world
			.get_resource_mut::<NextState<MouseContext>>()
			.unwrap()
			.set(MouseContext::Primed(KeyCode::KeyB));

		app.add_systems(Update, advance_just_released_mouse_context);
		app.update();
		app.update();

		let mouse_context = app
			.world
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::Primed(KeyCode::KeyB), mouse_context);
	}
}
