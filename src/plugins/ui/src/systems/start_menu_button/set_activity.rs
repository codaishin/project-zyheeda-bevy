use crate::components::{start_menu_button::StartMenuButton, ui_disabled::UIDisabled};
use bevy::prelude::*;
use common::prelude::*;

impl StartMenuButton {
	pub(crate) fn set_activity(
		target_trigger_state: GameStateCommand,
	) -> impl Fn(In<UIActivity>, ZyheedaCommands, Query<(Entity, &StartMenuButton)>) {
		move |In(activity), mut commands, buttons| {
			for (entity, StartMenuButton { trigger_state, .. }) in buttons {
				if trigger_state != &target_trigger_state {
					continue;
				}

				match activity {
					UIActivity::Enable => commands.try_apply_on(&entity, |mut e| {
						e.try_remove::<UIDisabled>();
					}),
					UIActivity::Disable => commands.try_apply_on(&entity, |mut e| {
						e.try_insert(UIDisabled);
					}),
				};
			}
		}
	}
}

pub(crate) enum UIActivity {
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
				StartMenuButton::triggers(GameStateCommand::NewGame),
			))
			.id();

		app.world_mut().run_system_once_with(
			StartMenuButton::set_activity(GameStateCommand::NewGame),
			UIActivity::Enable,
		)?;

		assert!(!app.world().entity(entity).contains::<UIDisabled>());
		Ok(())
	}

	#[test]
	fn disable() -> Result<(), RunSystemError> {
		let mut app = setup();
		let entity = app
			.world_mut()
			.spawn(StartMenuButton::triggers(GameStateCommand::NewGame))
			.id();

		app.world_mut().run_system_once_with(
			StartMenuButton::set_activity(GameStateCommand::NewGame),
			UIActivity::Disable,
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
				StartMenuButton::triggers(GameStateCommand::NewGame),
			))
			.id();

		app.world_mut().run_system_once_with(
			StartMenuButton::set_activity(GameStateCommand::Play),
			UIActivity::Enable,
		)?;

		assert!(app.world().entity(entity).contains::<UIDisabled>());
		Ok(())
	}
}
