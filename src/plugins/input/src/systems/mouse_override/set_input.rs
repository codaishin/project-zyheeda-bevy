use crate::resources::mouse_override::MouseOverride;
use bevy::{prelude::*, ui::Pressed};
use common::prelude::*;
use std::ops::Deref;

impl MouseOverride {
	pub(crate) fn set_input(
		mouse: Res<ButtonInput<MouseButton>>,
		buttons: Query<(), With<Pressed>>,
		mut mouse_override: ResMut<Self>,
	) {
		Self::reset_released(&mut mouse_override);

		let MouseOverride::Active { panel, action, .. } = *mouse_override else {
			return;
		};

		if !buttons.is_empty() {
			return;
		}

		let input = if mouse.just_released(MouseButton::Left) {
			InputState::just_released()
		} else if mouse.just_pressed(MouseButton::Left) {
			InputState::just_pressed()
		} else if mouse.pressed(MouseButton::Left) {
			InputState::pressed()
		} else {
			return;
		};

		*mouse_override = MouseOverride::Active {
			panel,
			action,
			input_state: Some(input),
		};
	}

	fn reset_released(mouse_override: &mut ResMut<MouseOverride>) {
		let &MouseOverride::Active {
			input_state: Some(InputState::Released { .. }),
			..
		} = (*mouse_override).deref()
		else {
			return;
		};

		**mouse_override = MouseOverride::Idle;
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::ui_widgets::Button;
	use testing::{SingleThreadedApp, set_input};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<ButtonInput<MouseButton>>();
		app.add_systems(Update, MouseOverride::set_input);

		app
	}

	mod bevy_input {
		use super::*;

		#[test]
		fn set_just_pressed() {
			let mut app = setup();
			let panel = app.world_mut().spawn(Button).id();
			app.insert_resource(MouseOverride::Active {
				panel,
				action: ActionKey::from(HandSlot::Left),
				input_state: None,
			});
			set_input!(app, just_pressed(MouseButton::Left));

			app.update();

			assert_eq!(
				&MouseOverride::Active {
					panel,
					action: ActionKey::from(HandSlot::Left),
					input_state: Some(InputState::just_pressed()),
				},
				app.world().resource::<MouseOverride>()
			);
		}

		#[test]
		fn set_pressed() {
			let mut app = setup();
			let panel = app.world_mut().spawn(Button).id();
			app.insert_resource(MouseOverride::Active {
				panel,
				action: ActionKey::from(HandSlot::Left),
				input_state: None,
			});
			set_input!(app, pressed(MouseButton::Left));

			app.update();

			assert_eq!(
				&MouseOverride::Active {
					panel,
					action: ActionKey::from(HandSlot::Left),
					input_state: Some(InputState::pressed()),
				},
				app.world().resource::<MouseOverride>()
			);
		}

		#[test]
		fn set_just_released() {
			let mut app = setup();
			let panel = app.world_mut().spawn(Button).id();
			app.insert_resource(MouseOverride::Active {
				panel,
				action: ActionKey::from(HandSlot::Left),
				input_state: None,
			});
			set_input!(app, just_released(MouseButton::Left));

			app.update();

			assert_eq!(
				&MouseOverride::Active {
					panel,
					action: ActionKey::from(HandSlot::Left),
					input_state: Some(InputState::just_released()),
				},
				app.world().resource::<MouseOverride>()
			);
		}

		#[test]
		fn do_set_when_panel_is_pressed() {
			let mut app = setup();
			let panel = app.world_mut().spawn((Button, Pressed)).id();
			app.insert_resource(MouseOverride::Active {
				panel,
				action: ActionKey::from(HandSlot::Left),
				input_state: None,
			});
			set_input!(app, just_pressed(MouseButton::Left));

			app.update();

			assert_eq!(
				&MouseOverride::Active {
					panel,
					action: ActionKey::from(HandSlot::Left),
					input_state: None,
				},
				app.world().resource::<MouseOverride>()
			);
		}

		#[test]
		fn do_set_when_other_is_pressed() {
			let mut app = setup();
			let panel = app.world_mut().spawn(Button).id();
			app.world_mut().spawn((Button, Pressed));
			app.insert_resource(MouseOverride::Active {
				panel,
				action: ActionKey::from(HandSlot::Left),
				input_state: None,
			});
			set_input!(app, just_pressed(MouseButton::Left));

			app.update();

			assert_eq!(
				&MouseOverride::Active {
					panel,
					action: ActionKey::from(HandSlot::Left),
					input_state: None,
				},
				app.world().resource::<MouseOverride>()
			);
		}
	}

	mod auto_idle {
		use super::*;

		#[test]
		fn set_just_released_to_idle() {
			let mut app = setup();
			let panel = app.world_mut().spawn(Button).id();
			app.insert_resource(MouseOverride::Active {
				panel,
				action: ActionKey::from(HandSlot::Left),
				input_state: Some(InputState::just_released()),
			});

			app.update();

			assert_eq!(
				&MouseOverride::Idle,
				app.world().resource::<MouseOverride>()
			);
		}

		#[test]
		fn set_released_to_idle() {
			let mut app = setup();
			let panel = app.world_mut().spawn(Button).id();
			app.insert_resource(MouseOverride::Active {
				panel,
				action: ActionKey::from(HandSlot::Left),
				input_state: Some(InputState::released()),
			});

			app.update();

			assert_eq!(
				&MouseOverride::Idle,
				app.world().resource::<MouseOverride>()
			);
		}
	}
}
