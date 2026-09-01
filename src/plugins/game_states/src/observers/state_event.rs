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
		next_state.set(on_state.state.into());
	}

	pub(crate) fn set_game_state_extension(
		on_state: On<StateEvent<GameStateCommandExtended<T>>>,
		mut commands: ZyheedaCommands,
		mut next_state: ResMut<NextState<CommandState<GameStateCommandExtended<T>>>>,
	) {
		use CommandState::*;
		use GameStateCommandExtended::*;

		let base = match on_state.state {
			Active(Command(cmd)) => Active(cmd),
			_ => Dirty,
		};

		commands.trigger_observers_for(StateEvent { state: base });
		next_state.set(on_state.state);
	}
}
