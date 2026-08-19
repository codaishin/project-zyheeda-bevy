use crate::resources::{
	game_state_context::GameStateContext,
	game_state_roles::{GAME_STATE_ROLES_DEFAULT, GameStateRoles},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::prelude::*;
use std::collections::HashSet;

#[derive(SystemParam)]
pub struct GameStatesRead<'w> {
	current: Res<'w, GameStateContext>,
}

impl GameStates for GameStatesRead<'_> {
	fn activity(&self) -> Activity {
		self.current.activity
	}

	fn ui(&self) -> &'_ HashSet<IngameUI> {
		&self.current.ui
	}
}

impl GamePaused for GameStatesRead<'static> {
	fn game_paused() -> impl IntoSystem<(), bool, (), System: ReadOnlySystem> {
		IntoSystem::into_system(
			|states: GameStatesRead, roles: Option<Res<GameStateRoles>>| {
				let roles = match roles {
					Some(r) => r.into_inner(),
					None => &GAME_STATE_ROLES_DEFAULT,
				};

				states.iter().any(|state| roles.is_pause_state(state))
			},
		)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::GameStatesPlugin;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use testing::SingleThreadedApp;

	fn setup<const N: usize>(activity: Activity, ui: [IngameUI; N]) -> App {
		let mut app = App::new().single_threaded(Update);

		app.insert_resource(GameStateContext {
			activity,
			ui: HashSet::from(ui),
		});

		app
	}

	#[test]
	fn is_paused() -> Result<(), RunSystemError> {
		let mut app = setup(Activity::Settable(SettableActivity::Paused), []);

		let paused = app
			.world_mut()
			.run_system_once(GameStatesRead::game_paused())?;

		assert!(paused);
		Ok(())
	}

	#[test]
	fn not_paused_on_play() -> Result<(), RunSystemError> {
		let mut app = setup(Activity::Settable(SettableActivity::Play), []);

		let paused = app
			.world_mut()
			.run_system_once(GameStatesRead::game_paused())?;

		assert!(!paused);
		Ok(())
	}

	#[test]
	fn not_paused_when_activity_marked_as_not_pausing() -> Result<(), RunSystemError> {
		let mut app = setup(Activity::Settable(SettableActivity::Paused), []);
		GameStatesPlugin::add_non_pause_state(&mut app, SettableActivity::Paused);

		let paused = app
			.world_mut()
			.run_system_once(GameStatesRead::game_paused())?;

		assert!(!paused);
		Ok(())
	}

	#[test]
	fn paused_on_hud() -> Result<(), RunSystemError> {
		let mut app = setup(Activity::Settable(SettableActivity::Play), [IngameUI::Hud]);

		let paused = app
			.world_mut()
			.run_system_once(GameStatesRead::game_paused())?;

		assert!(paused);
		Ok(())
	}

	#[test]
	fn not_paused_when_ui_marked_as_not_pausing() -> Result<(), RunSystemError> {
		let mut app = setup(Activity::Settable(SettableActivity::Play), [IngameUI::Hud]);
		GameStatesPlugin::add_non_pause_state(&mut app, IngameUI::Hud);

		let paused = app
			.world_mut()
			.run_system_once(GameStatesRead::game_paused())?;

		assert!(!paused);
		Ok(())
	}
}
