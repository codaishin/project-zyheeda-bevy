use crate::components::{start_menu_button::StartMenuButton, ui_disabled::UIDisabled};
use bevy::prelude::*;
use common::{
	states::game_state::GameState,
	traits::{try_insert_on::TryInsertOn, try_remove_from::TryRemoveFrom},
};

impl StartMenuButton {
	pub(crate) fn set_activity(
		target_trigger_state: GameState,
	) -> impl Fn(In<Activity>, Commands, Query<(Entity, &StartMenuButton)>) {
		move |In(activity), mut commands, buttons| {
			for (entity, StartMenuButton { trigger_state, .. }) in buttons {
				if trigger_state != &target_trigger_state {
					continue;
				}

				match activity {
					Activity::Enable => commands.try_remove_from::<UIDisabled>(entity),
					Activity::Disable => commands.try_insert_on(entity, UIDisabled),
				};
			}
		}
	}
}

pub(crate) enum Activity {
	Enable,
	Disable,
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::components::ui_disabled::UIDisabled;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		App::new().single_threaded(Update)
	}

	#[test]
	fn enable() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				UIDisabled,
				StartMenuButton {
					trigger_state: GameState::NewGame,
					..default()
				},
			))
			.id();

		app.world_mut().run_system_once_with(
			StartMenuButton::set_activity(GameState::NewGame),
			Activity::Enable,
		)?;

		assert!(!app.world().entity(entity).contains::<UIDisabled>());
		Ok(())
	}

	#[test]
	fn disable() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((StartMenuButton {
				trigger_state: GameState::NewGame,
				..default()
			},))
			.id();

		app.world_mut().run_system_once_with(
			StartMenuButton::set_activity(GameState::NewGame),
			Activity::Disable,
		)?;

		assert!(app.world().entity(entity).contains::<UIDisabled>());
		Ok(())
	}

	#[test]
	fn ignore_on_trigger_state_mismatch() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn((
				UIDisabled,
				StartMenuButton {
					trigger_state: GameState::NewGame,
					..default()
				},
			))
			.id();

		app.world_mut().run_system_once_with(
			StartMenuButton::set_activity(GameState::Play),
			Activity::Enable,
		)?;

		assert!(app.world().entity(entity).contains::<UIDisabled>());
		Ok(())
	}
}
