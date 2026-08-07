use crate::{events::GameStateEvent, resources::game_state_context::GameStatesContext};
use bevy::{ecs::system::SystemParam, prelude::*};
use common::{
	traits::handles_game_states::{GameState, GameStates, GameStatesMut},
	zyheeda_commands::ZyheedaCommands,
};
use zyheeda_core::collections::ordered::OrderedSet;

#[derive(SystemParam)]
pub struct GameStatesWrite<'w, 's> {
	commands: ZyheedaCommands<'w, 's>,
	ctx: ResMut<'w, GameStatesContext>,
}

impl GameStates for GameStatesWrite<'_, '_> {
	fn game_states(&self) -> &OrderedSet<GameState> {
		&self.ctx.states
	}
}

impl GameStatesMut for GameStatesWrite<'_, '_> {
	fn game_states_mut(&mut self) -> &mut OrderedSet<GameState> {
		&mut self.ctx.states
	}
}

impl Drop for GameStatesWrite<'_, '_> {
	fn drop(&mut self) {
		for event in self.ctx.states.iter() {
			self.commands.trigger_observers_for(GameStateEvent(*event));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::events::GameStateEvent;
	use bevy::ecs::system::{RunSystemError, RunSystemOnce};
	use common::traits::handles_game_states::MenuState;
	use testing::SingleThreadedApp;

	#[derive(Resource, Debug, PartialEq, Default)]
	struct Events(Vec<GameStateEvent>);

	impl Events {
		fn record(on_event: On<GameStateEvent>, mut events: ResMut<Events>) {
			events.0.push(*on_event.event());
		}
	}

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.init_resource::<Events>();
		app.init_resource::<GameStatesContext>();
		app.add_observer(Events::record);

		app
	}

	#[test]
	fn fire_game_state_event() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut().run_system_once(|mut g: GameStatesWrite| {
			g.game_states_mut()
				.insert(GameState::Menu(MenuState::Settings));
		})?;

		assert_eq!(
			&Events(vec![GameStateEvent(GameState::Menu(MenuState::Settings))]),
			app.world().resource::<Events>()
		);
		Ok(())
	}

	#[test]
	fn only_fire_last_event() -> Result<(), RunSystemError> {
		let mut app = setup();

		app.world_mut().run_system_once(|mut g: GameStatesWrite| {
			let states = g.game_states_mut();
			states.insert(GameState::Menu(MenuState::Settings));
			states.insert(GameState::Menu(MenuState::Inventory));
		})?;

		assert_eq!(
			&Events(vec![GameStateEvent(GameState::Menu(MenuState::Inventory))]),
			app.world().resource::<Events>()
		);
		Ok(())
	}
}
