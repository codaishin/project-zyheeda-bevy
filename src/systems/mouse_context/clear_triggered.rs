use crate::states::MouseContext;
use bevy::ecs::{
	schedule::{NextState, State},
	system::{Res, ResMut},
};

pub fn clear_triggered_mouse_context(
	mouse_context: Res<State<MouseContext>>,
	mut next_mouse_context: ResMut<NextState<MouseContext>>,
) {
	let MouseContext::Triggered(_) = mouse_context.get() else {
		return;
	};
	next_mouse_context.set(MouseContext::Default);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::states::MouseContext;
	use bevy::{
		app::{App, Update},
		ecs::schedule::{NextState, State},
		input::keyboard::KeyCode,
	};

	#[test]
	fn reset_on_any_key_press() {
		let mut app = App::new();

		app.add_state::<MouseContext>();
		app.world
			.get_resource_mut::<NextState<MouseContext>>()
			.unwrap()
			.set(MouseContext::Triggered(KeyCode::B));

		app.add_systems(Update, clear_triggered_mouse_context);
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
	fn no_reset_when_no_key_pressed() {
		let mut app = App::new();

		app.add_state::<MouseContext>();
		app.world
			.get_resource_mut::<NextState<MouseContext>>()
			.unwrap()
			.set(MouseContext::Primed(KeyCode::B));

		app.add_systems(Update, clear_triggered_mouse_context);
		app.update();
		app.update();

		let mouse_context = app
			.world
			.get_resource::<State<MouseContext>>()
			.unwrap()
			.get();

		assert_eq!(&MouseContext::Primed(KeyCode::B), mouse_context);
	}
}
