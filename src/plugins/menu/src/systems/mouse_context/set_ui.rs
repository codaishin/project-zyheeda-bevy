use bevy::{
	ecs::system::{Query, Res, ResMut},
	state::state::{NextState, State},
	ui::Interaction,
};
use common::states::mouse_context::MouseContext;

pub fn set_ui_mouse_context(
	current_state: Res<State<MouseContext>>,
	mut next_state: ResMut<NextState<MouseContext>>,
	interactions: Query<&Interaction>,
) {
	if primed_or_triggered(current_state.get()) {
		return;
	}

	if next_already_set(&next_state) {
		return;
	}

	next_state.set(match interactions.iter().all(is_none) {
		true => MouseContext::Default,
		false => MouseContext::UI,
	});
}

fn primed_or_triggered(context: &MouseContext) -> bool {
	matches!(
		context,
		MouseContext::Primed(_) | MouseContext::Triggered(_) | MouseContext::JustTriggered(_)
	)
}

fn next_already_set(next_state: &ResMut<NextState<MouseContext>>) -> bool {
	matches!(next_state.as_ref() , NextState::Pending(next_state) if next_state != &MouseContext::Default)
}

fn is_none(interaction: &Interaction) -> bool {
	interaction == &Interaction::None
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::{
		app::{App, Update},
		ecs::schedule::IntoSystemConfigs,
		input::keyboard::KeyCode,
		state::app::{AppExtStates, StatesPlugin},
		ui::Interaction,
	};
	use common::tools::action_key::user_input::UserInput;

	fn setup() -> App {
		let mut app = App::new();

		app.add_plugins(StatesPlugin);
		app.init_state::<MouseContext>();
		app.add_systems(Update, set_ui_mouse_context);

		app
	}

	fn setup_with_next_mouse_context(mouse_context: MouseContext) -> App {
		let mut app = App::new();

		app.add_plugins(StatesPlugin);
		app.init_state::<MouseContext>();

		let set_next = move |mut next: ResMut<NextState<MouseContext>>| {
			next.set(mouse_context.clone());
		};

		app.add_systems(Update, (set_next, set_ui_mouse_context).chain());

		app
	}

	#[test]
	fn set_context_ui_when_some_interaction_hovered() {
		let mut app = setup();

		app.world_mut().spawn(Interaction::Hovered);
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Default);

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::UI,
			app.world()
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}

	#[test]
	fn set_context_ui_when_some_interaction_pressed() {
		let mut app = setup();

		app.world_mut().spawn(Interaction::Pressed);
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Default);

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::UI,
			app.world()
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}

	#[test]
	fn set_context_default_when_no_interaction() {
		let mut app = setup();

		app.world_mut().spawn(Interaction::None);
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::UI);

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::Default,
			app.world()
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}

	#[test]
	fn ignore_when_mouse_context_is_mouse_context_primed() {
		let mut app = setup();

		app.world_mut().spawn(Interaction::None);
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Primed(UserInput::from(KeyCode::KeyA)));

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::Primed(UserInput::from(KeyCode::KeyA)),
			app.world()
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}

	#[test]
	fn ignore_when_mouse_context_is_mouse_context_just_triggered() {
		let mut app = setup();

		app.world_mut().spawn(Interaction::None);
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::JustTriggered(UserInput::from(KeyCode::KeyA)));

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::JustTriggered(UserInput::from(KeyCode::KeyA)),
			app.world()
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}

	#[test]
	fn ignore_when_mouse_context_is_mouse_context_triggered() {
		let mut app = setup();

		app.world_mut().spawn(Interaction::None);
		app.world_mut()
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Triggered(UserInput::from(KeyCode::KeyA)));

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::Triggered(UserInput::from(KeyCode::KeyA)),
			app.world()
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}

	#[test]
	fn ignore_when_next_state_is_already_set() {
		let mut app =
			setup_with_next_mouse_context(MouseContext::Primed(UserInput::from(KeyCode::KeyA)));

		app.world_mut().spawn(Interaction::Hovered);

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::Primed(UserInput::from(KeyCode::KeyA)),
			app.world()
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}

	#[test]
	fn wet_when_next_state_is_set_to_default() {
		let mut app = setup_with_next_mouse_context(MouseContext::Default);

		app.world_mut().spawn(Interaction::Hovered);

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::UI,
			app.world()
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}
}
