use std::ops::Deref;

use crate::{
	components::action_key_interaction::ActionKeyInteraction,
	resources::mouse_override::MouseOverride,
};
use bevy::prelude::*;
use common::traits::handles_input::InputState;

impl ActionKeyInteraction {
	pub(crate) fn set_mouse_override_from_mouse(
		mouse: Res<ButtonInput<MouseButton>>,
		interactions: Query<&ActionKeyInteraction>,
		mut mouse_override: ResMut<MouseOverride>,
	) {
		Self::just_released_to_idle(&mut mouse_override);
		Self::missing_panel_fallback(&interactions, &mut mouse_override);

		let panel = match *mouse_override {
			MouseOverride::Ui { panel } => panel,
			MouseOverride::World { panel, .. } => panel,
			_ => return,
		};

		let Ok(Self { action_key, .. }) = interactions.get(panel) else {
			return;
		};

		let input = if mouse.just_released(MOUSE_LEFT) {
			InputState::just_released()
		} else if mouse.just_pressed(MOUSE_LEFT) {
			InputState::just_pressed()
		} else if mouse.pressed(MOUSE_LEFT) {
			InputState::pressed()
		} else {
			return;
		};

		*mouse_override = MouseOverride::World {
			panel,
			action: *action_key,
			input_state: input,
		};
	}

	fn just_released_to_idle(mouse_override: &mut ResMut<MouseOverride>) {
		let &MouseOverride::World {
			input_state: InputState::Released { just_now: true },
			..
		} = (*mouse_override).deref()
		else {
			return;
		};

		**mouse_override = MouseOverride::Idle;
	}

	fn missing_panel_fallback(
		interactions: &Query<&ActionKeyInteraction>,
		mouse_override: &mut ResMut<MouseOverride>,
	) {
		match (*mouse_override).deref() {
			MouseOverride::Idle => {}
			MouseOverride::Ui { panel } => {
				let Err(_) = interactions.get(*panel) else {
					return;
				};

				**mouse_override = MouseOverride::Idle;
			}
			MouseOverride::World { panel, action, .. } => {
				let Err(_) = interactions.get(*panel) else {
					return;
				};

				**mouse_override = MouseOverride::World {
					panel: *panel,
					action: *action,
					input_state: InputState::just_released(),
				};
			}
		}
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
		app.add_systems(Update, ActionKeyInteraction::set_mouse_override_from_mouse);

		app
	}

	mod from_ui {
		use super::*;

		#[test]
		fn set_just_pressed() {
			let mut app = setup();
			let panel = app
				.world_mut()
				.spawn(ActionKeyInteraction {
					action_key: ActionKey::from(PlayerSlot::LOWER_L),
					override_active: true,
				})
				.id();
			app.insert_resource(MouseOverride::Ui { panel });
			set_input!(app, just_pressed(MOUSE_LEFT));

			app.update();

			assert_eq!(
				&MouseOverride::World {
					panel,
					action: ActionKey::from(PlayerSlot::LOWER_L),
					input_state: InputState::just_pressed()
				},
				app.world().resource::<MouseOverride>()
			);
		}

		#[test]
		fn set_pressed() {
			let mut app = setup();
			let panel = app
				.world_mut()
				.spawn(ActionKeyInteraction {
					action_key: ActionKey::from(PlayerSlot::LOWER_L),
					override_active: true,
				})
				.id();
			app.insert_resource(MouseOverride::Ui { panel });
			set_input!(app, pressed(MOUSE_LEFT));

			app.update();

			assert_eq!(
				&MouseOverride::World {
					panel,
					action: ActionKey::from(PlayerSlot::LOWER_L),
					input_state: InputState::pressed()
				},
				app.world().resource::<MouseOverride>()
			);
		}

		#[test]
		fn set_just_released() {
			let mut app = setup();
			let panel = app
				.world_mut()
				.spawn(ActionKeyInteraction {
					action_key: ActionKey::from(PlayerSlot::LOWER_L),
					override_active: true,
				})
				.id();
			app.insert_resource(MouseOverride::Ui { panel });
			set_input!(app, just_released(MOUSE_LEFT));

			app.update();

			assert_eq!(
				&MouseOverride::World {
					panel,
					action: ActionKey::from(PlayerSlot::LOWER_L),
					input_state: InputState::just_released()
				},
				app.world().resource::<MouseOverride>()
			);
		}
	}

	mod from_world {
		use super::*;

		#[test]
		fn set_just_pressed() {
			let mut app = setup();
			let panel = app
				.world_mut()
				.spawn(ActionKeyInteraction {
					action_key: ActionKey::from(PlayerSlot::LOWER_L),
					override_active: true,
				})
				.id();
			app.insert_resource(MouseOverride::World {
				panel,
				action: ActionKey::default(),
				input_state: InputState::pressed(),
			});
			set_input!(app, just_pressed(MOUSE_LEFT));

			app.update();

			assert_eq!(
				&MouseOverride::World {
					panel,
					action: ActionKey::from(PlayerSlot::LOWER_L),
					input_state: InputState::just_pressed()
				},
				app.world().resource::<MouseOverride>()
			);
		}

		#[test]
		fn set_pressed() {
			let mut app = setup();
			let panel = app
				.world_mut()
				.spawn(ActionKeyInteraction {
					action_key: ActionKey::from(PlayerSlot::LOWER_L),
					override_active: true,
				})
				.id();
			app.insert_resource(MouseOverride::World {
				panel,
				action: ActionKey::default(),
				input_state: InputState::just_pressed(),
			});
			set_input!(app, pressed(MOUSE_LEFT));

			app.update();

			assert_eq!(
				&MouseOverride::World {
					panel,
					action: ActionKey::from(PlayerSlot::LOWER_L),
					input_state: InputState::pressed()
				},
				app.world().resource::<MouseOverride>()
			);
		}

		#[test]
		fn set_just_released() {
			let mut app = setup();
			let panel = app
				.world_mut()
				.spawn(ActionKeyInteraction {
					action_key: ActionKey::from(PlayerSlot::LOWER_L),
					override_active: true,
				})
				.id();
			app.insert_resource(MouseOverride::World {
				panel,
				action: ActionKey::default(),
				input_state: InputState::pressed(),
			});
			set_input!(app, just_released(MOUSE_LEFT));

			app.update();

			assert_eq!(
				&MouseOverride::World {
					panel,
					action: ActionKey::from(PlayerSlot::LOWER_L),
					input_state: InputState::just_released()
				},
				app.world().resource::<MouseOverride>()
			);
		}
	}

	mod fallback_when_panel_not_found {
		use super::*;

		#[test]
		fn set_to_just_released_from_world() {
			let mut app = setup();
			app.insert_resource(MouseOverride::World {
				panel: Entity::from_raw(123),
				action: ActionKey::from(PlayerSlot::UPPER_L),
				input_state: InputState::just_pressed(),
			});

			app.update();

			assert_eq!(
				&MouseOverride::World {
					panel: Entity::from_raw(123),
					action: ActionKey::from(PlayerSlot::UPPER_L),
					input_state: InputState::just_released()
				},
				app.world().resource::<MouseOverride>()
			);
		}

		#[test]
		fn set_to_idle_from_ui() {
			let mut app = setup();
			app.insert_resource(MouseOverride::Ui {
				panel: Entity::from_raw(123),
			});

			app.update();

			assert_eq!(
				&MouseOverride::Idle,
				app.world().resource::<MouseOverride>()
			);
		}
	}

	mod auto_idle {
		use super::*;

		#[test]
		fn set_to_idle_after_just_released() {
			let mut app = setup();
			let panel = app
				.world_mut()
				.spawn(ActionKeyInteraction {
					action_key: ActionKey::from(PlayerSlot::LOWER_L),
					override_active: true,
				})
				.id();
			app.insert_resource(MouseOverride::World {
				panel,
				action: ActionKey::from(PlayerSlot::UPPER_L),
				input_state: InputState::just_released(),
			});

			app.update();

			assert_eq!(
				&MouseOverride::Idle,
				app.world().resource::<MouseOverride>()
			);
		}
	}
}
