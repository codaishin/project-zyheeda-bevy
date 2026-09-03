use crate::{events::StateEvent, states::command_state::CommandState};
use bevy::prelude::*;
use common::prelude::*;
use std::{fmt::Debug, hash::Hash};

impl StateEvent<GameStateCommand> {
	pub(crate) fn set_game_state(
		on_state: On<StateEvent<GameStateCommand>>,
		mut next_state: ResMut<NextState<CommandState>>,
	) {
		next_state.set(on_state.state);
	}
}

impl<T> StateEvent<GameStateCommandExtended<T>>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	pub(crate) fn set_game_state(
		on_state: On<StateEvent<GameStateCommand>>,
		mut next_state: ResMut<NextState<CommandState<GameStateCommandExtended<T>>>>,
	) {
		let next = match on_state.state.try_into_active() {
			Some(cmd) => CommandState::active(GameStateCommandExtended::Command(cmd)),
			_ => CommandState::dirty(),
		};

		next_state.set(next);
	}

	pub(crate) fn set_game_state_extension(
		on_state: On<StateEvent<GameStateCommandExtended<T>>>,
		mut commands: ZyheedaCommands,
		mut next_state: ResMut<NextState<CommandState<GameStateCommandExtended<T>>>>,
	) {
		let base = match on_state.state.try_into_active() {
			Some(GameStateCommandExtended::Command(cmd)) => CommandState::active(cmd),
			_ => CommandState::dirty(),
		};

		commands.trigger_observers_for(StateEvent { state: base });
		next_state.set(on_state.state);
	}
}
