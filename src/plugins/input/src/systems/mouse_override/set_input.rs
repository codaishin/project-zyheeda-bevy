use crate::resources::mouse_override::MouseOverride;
use bevy::prelude::*;
use common::traits::handles_input::InputState;
use std::ops::Deref;

impl MouseOverride {
	pub(crate) fn set_input(
		mouse: Res<ButtonInput<MouseButton>>,
		interactions: Query<&Interaction>,
		mut mouse_override: ResMut<Self>,
	) {
		Self::reset_released(&mut mouse_override);

		let MouseOverride::Active { panel, action, .. } = *mouse_override else {
			return;
		};

		if interactions.iter().any(|i| i != &Interaction::None) {
			return;
		}

		let input = if mouse.just_released(MOUSE_LEFT) {
			InputState::just_released()
		} else if mouse.just_pressed(MOUSE_LEFT) {
			InputState::just_pressed()
		} else if mouse.pressed(MOUSE_LEFT) {
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

const MOUSE_LEFT: MouseButton = MouseButton::Left;

#[cfg(test)]
mod tests {
	use super::*;
	use common::tools::action_key::{ActionKey, slot::PlayerSlot};
	use testing::{SingleThreadedApp, set_input};

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<ButtonInput<MouseButton>>();
		app.add_systems(Update, MouseOverride::set_input);

		app
	}

	mod bevy_input {
		use super::*;
		use test_case::test_case;

		#[test]
		fn set_just_pressed() {
			let mut app = setup();
			let panel = app.world_mut().spawn(Interaction::None).id();
			app.insert_resource(MouseOverride::Active {
				panel,
				action: ActionKey::from(PlayerSlot::LOWER_L),
				input_state: None,
			});
			set_input!(app, just_pressed(MOUSE_LEFT));

			app.update();

			assert_eq!(
				&MouseOverride::Active {
					panel,
					action: ActionKey::from(PlayerSlot::LOWER_L),
					input_state: Some(InputState::just_pressed()),
				},
				app.world().resource::<MouseOverride>()
			);
		}

		#[test]
		fn set_pressed() {
			let mut app = setup();
			let panel = app.world_mut().spawn(Interaction::None).id();
			app.insert_resource(MouseOverride::Active {
				panel,
				action: ActionKey::from(PlayerSlot::LOWER_L),
				input_state: None,
			});
			set_input!(app, pressed(MOUSE_LEFT));

			app.update();

			assert_eq!(
				&MouseOverride::Active {
					panel,
					action: ActionKey::from(PlayerSlot::LOWER_L),
					input_state: Some(InputState::pressed()),
				},
				app.world().resource::<MouseOverride>()
			);
		}

		#[test]
		fn set_just_released() {
			let mut app = setup();
			let panel = app.world_mut().spawn(Interaction::None).id();
			app.insert_resource(MouseOverride::Active {
				panel,
				action: ActionKey::from(PlayerSlot::LOWER_L),
				input_state: None,
			});
			set_input!(app, just_released(MOUSE_LEFT));

			app.update();

			assert_eq!(
				&MouseOverride::Active {
					panel,
					action: ActionKey::from(PlayerSlot::LOWER_L),
					input_state: Some(InputState::just_released()),
				},
				app.world().resource::<MouseOverride>()
			);
		}

		#[test_case(Interaction::Hovered; "hovered")]
		#[test_case(Interaction::Pressed; "pressed")]
		fn do_set_when_panel_is(interaction: Interaction) {
			let mut app = setup();
			let panel = app.world_mut().spawn(interaction).id();
			app.insert_resource(MouseOverride::Active {
				panel,
				action: ActionKey::from(PlayerSlot::LOWER_L),
				input_state: None,
			});
			set_input!(app, just_pressed(MOUSE_LEFT));

			app.update();

			assert_eq!(
				&MouseOverride::Active {
					panel,
					action: ActionKey::from(PlayerSlot::LOWER_L),
					input_state: None,
				},
				app.world().resource::<MouseOverride>()
			);
		}

		#[test_case(Interaction::Hovered; "hovered")]
		#[test_case(Interaction::Pressed; "pressed")]
		fn do_set_when_other_is(interaction: Interaction) {
			let mut app = setup();
			let panel = app.world_mut().spawn(Interaction::None).id();
			app.world_mut().spawn(interaction);
			app.insert_resource(MouseOverride::Active {
				panel,
				action: ActionKey::from(PlayerSlot::LOWER_L),
				input_state: None,
			});
			set_input!(app, just_pressed(MOUSE_LEFT));

			app.update();

			assert_eq!(
				&MouseOverride::Active {
					panel,
					action: ActionKey::from(PlayerSlot::LOWER_L),
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
			let panel = app.world_mut().spawn(Interaction::None).id();
			app.insert_resource(MouseOverride::Active {
				panel,
				action: ActionKey::from(PlayerSlot::UPPER_L),
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
			let panel = app.world_mut().spawn(Interaction::None).id();
			app.insert_resource(MouseOverride::Active {
				panel,
				action: ActionKey::from(PlayerSlot::UPPER_L),
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
