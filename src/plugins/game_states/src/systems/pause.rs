use crate::{
	resources::game_state_roles::GameStateRoles,
	system_params::game_states_read::GameStatesRead,
};
use bevy::prelude::*;
use common::prelude::*;

impl GameStateRoles {
	pub(crate) fn pause(
		mut time: ResMut<Time<Virtual>>,
		current: GameStatesRead,
		roles: Res<GameStateRoles>,
	) {
		if current.iter().any(|state| roles.is_pause_state(state)) {
			time.pause();
		} else {
			time.unpause();
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		GameStatesPlugin,
		resources::game_state_context::GameStateContext,
		states::activity::Activity,
		system_params::ui_states::UIStates,
	};
	use bevy::{state::app::StatesPlugin, time::TimePlugin};
	use testing::SingleThreadedApp;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		app.add_plugins(TimePlugin);
		UIStates::init(&mut app);
		app.init_state::<Activity>();
		app.init_resource::<GameStateContext>();
		app.init_resource::<GameStateRoles>();
		app.add_systems(Update, GameStateRoles::pause);

		app
	}

	#[test]
	fn pause() {
		let mut app = setup();
		app.world_mut().resource_mut::<GameStateContext>().activity = ActivityState::Paused;

		app.update();

		assert!(app.world().resource::<Time<Virtual>>().is_paused());
	}

	#[test]
	fn un_pause_if_state_excluded_from_pause() {
		let mut app = setup();
		app.world_mut().resource_mut::<Time<Virtual>>().pause();
		app.world_mut()
			.resource_mut::<GameStateRoles>()
			.non_pause_states
			.insert(GameState::Activity(ActivityState::Paused));
		app.world_mut().resource_mut::<GameStateContext>().activity = ActivityState::Paused;

		app.update();

		assert!(!app.world().resource::<Time<Virtual>>().is_paused());
	}

	#[test]
	fn default_non_pause_states_do_not_pause() {
		let mut app = setup();

		for state in GameStatesPlugin::DEFAULT {
			app.world_mut().resource_mut::<Time<Virtual>>().pause();
			app.world_mut().resource_mut::<GameStateContext>().activity = *state;

			app.update();

			assert!(!app.world().resource::<Time<Virtual>>().is_paused());
		}
	}

	#[test]
	fn pause_on_ui_state() {
		let mut app = setup();
		app.world_mut()
			.resource_mut::<GameStateRoles>()
			.non_pause_states
			.insert(GameState::Activity(ActivityState::Paused));
		app.world_mut().resource_mut::<GameStateContext>().activity = ActivityState::Paused;
		app.world_mut()
			.resource_mut::<GameStateContext>()
			.ui
			.insert(UIState::Hud);

		app.update();

		assert!(app.world().resource::<Time<Virtual>>().is_paused());
	}

	#[test]
	fn un_pause_if_ui_state_excluded_from_pause() {
		let mut app = setup();
		app.world_mut().resource_mut::<Time<Virtual>>().pause();
		app.world_mut()
			.resource_mut::<GameStateRoles>()
			.non_pause_states
			.extend([
				GameState::Activity(ActivityState::Paused),
				GameState::IngameUI(UIState::Hud),
			]);
		app.world_mut().resource_mut::<GameStateContext>().activity = ActivityState::Paused;
		app.world_mut()
			.resource_mut::<GameStateContext>()
			.ui
			.insert(UIState::Hud);

		app.update();

		assert!(!app.world().resource::<Time<Virtual>>().is_paused());
	}
}
