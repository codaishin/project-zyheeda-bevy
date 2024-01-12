use crate::states::MouseContext;
use bevy::{
	ecs::{
		schedule::{NextState, State},
		system::{Query, Res, ResMut},
	},
	ui::Interaction,
};

pub fn set_ui_mouse_context(
	current_state: Res<State<MouseContext>>,
	mut next_state: ResMut<NextState<MouseContext>>,
	interactions: Query<&Interaction>,
) {
	if let MouseContext::Primed(_) | MouseContext::Triggered(_) = current_state.get() {
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

fn next_already_set(next_state: &ResMut<NextState<MouseContext>>) -> bool {
	matches!(&next_state.0, Some(next_state) if next_state != &MouseContext::Default)
}

fn is_none(interaction: &Interaction) -> bool {
	interaction == &Interaction::None
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::states::MouseContext;
	use bevy::{
		app::{App, Update},
		ecs::schedule::{IntoSystemConfigs, State},
		input::keyboard::KeyCode,
		ui::Interaction,
	};

	#[test]
	fn set_context_ui_when_some_interaction_hovered() {
		let mut app = App::new();

		app.add_systems(Update, set_ui_mouse_context);
		app.add_state::<MouseContext>();
		app.world.spawn(Interaction::Hovered);
		app.world
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Default);

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::UI,
			app.world
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}

	#[test]
	fn set_context_ui_when_some_interaction_pressed() {
		let mut app = App::new();

		app.add_systems(Update, set_ui_mouse_context);
		app.add_state::<MouseContext>();
		app.world.spawn(Interaction::Pressed);
		app.world
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Default);

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::UI,
			app.world
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}

	#[test]
	fn set_context_default_when_no_interaction() {
		let mut app = App::new();

		app.add_systems(Update, set_ui_mouse_context);
		app.add_state::<MouseContext>();
		app.world.spawn(Interaction::None);
		app.world
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::UI);

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::Default,
			app.world
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}

	#[test]
	fn ignore_when_mouse_context_is_mouse_context_primed() {
		let mut app = App::new();

		app.add_systems(Update, set_ui_mouse_context);
		app.add_state::<MouseContext>();
		app.world.spawn(Interaction::None);
		app.world
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Primed(KeyCode::A));

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::Primed(KeyCode::A),
			app.world
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}

	#[test]
	fn ignore_when_mouse_context_is_mouse_context_triggered() {
		let mut app = App::new();

		app.add_systems(Update, set_ui_mouse_context);
		app.add_state::<MouseContext>();
		app.world.spawn(Interaction::None);
		app.world
			.resource_mut::<NextState<MouseContext>>()
			.set(MouseContext::Triggered(KeyCode::A));

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::Triggered(KeyCode::A),
			app.world
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}

	#[test]
	fn ignore_when_next_state_is_already_set() {
		let mut app = App::new();

		fn set_next(mut next: ResMut<NextState<MouseContext>>) {
			next.set(MouseContext::Primed(KeyCode::A));
		}

		app.add_systems(Update, (set_next, set_ui_mouse_context).chain());
		app.add_state::<MouseContext>();
		app.world.spawn(Interaction::Hovered);

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::Primed(KeyCode::A),
			app.world
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}

	#[test]
	fn wet_when_next_state_is_set_to_default() {
		let mut app = App::new();

		fn set_next(mut next: ResMut<NextState<MouseContext>>) {
			next.set(MouseContext::Default);
		}

		app.add_systems(Update, (set_next, set_ui_mouse_context).chain());
		app.add_state::<MouseContext>();
		app.world.spawn(Interaction::Hovered);

		app.update();
		app.update();

		assert_eq!(
			&MouseContext::UI,
			app.world
				.get_resource::<State<MouseContext>>()
				.unwrap()
				.get()
		);
	}
}
