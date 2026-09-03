use crate::{events::StateEvent, states::command_state::CommandState};
use bevy::prelude::*;
use common::prelude::*;
use std::{any::TypeId, fmt::Debug, hash::Hash};

impl StateEvent<GameStateCommand> {
	pub(crate) fn set_game_state(
		on_state: On<StateEvent<GameStateCommand>>,
		mut next_state: ResMut<NextState<CommandState>>,
	) {
		next_state.set((*on_state).into());
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
		let next = match *on_state {
			StateEvent::Active(cmd) => CommandState::active(GameStateCommandExtended::Command(cmd)),
			StateEvent::Dirty { issued_by } if issued_by != Self::id() => CommandState::dirty(),
			_ => return,
		};

		next_state.set(next);
	}

	pub(crate) fn set_game_state_extension(
		on_state: On<StateEvent<GameStateCommandExtended<T>>>,
		mut commands: ZyheedaCommands,
		mut next_state: ResMut<NextState<CommandState<GameStateCommandExtended<T>>>>,
	) {
		let base = match *on_state {
			StateEvent::Active(GameStateCommandExtended::Command(cmd)) => StateEvent::Active(cmd),
			_ => StateEvent::Dirty {
				issued_by: Self::id(),
			},
		};

		commands.trigger_observers_for(base);
		next_state.set((*on_state).into());
	}

	fn id() -> TypeId {
		TypeId::of::<GameStateCommandExtended<T>>()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy::state::app::StatesPlugin;
	use testing::SingleThreadedApp;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _A;

	#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
	struct _B;

	fn setup() -> App {
		let mut app = App::new().single_threaded(Update);

		app.add_plugins(StatesPlugin);
		app.init_state::<CommandState<GameStateCommand>>();
		app.add_observer(StateEvent::<GameStateCommand>::set_game_state);

		app.init_state::<CommandState<GameStateCommandExtended<_A>>>();
		app.add_observer(StateEvent::<GameStateCommandExtended<_A>>::set_game_state);
		app.add_observer(StateEvent::<GameStateCommandExtended<_A>>::set_game_state_extension);

		app.init_state::<CommandState<GameStateCommandExtended<_B>>>();
		app.add_observer(StateEvent::<GameStateCommandExtended<_B>>::set_game_state);
		app.add_observer(StateEvent::<GameStateCommandExtended<_B>>::set_game_state_extension);

		app
	}

	#[test]
	fn sync_base_state() {
		let mut app = setup();

		app.world_mut()
			.trigger(StateEvent::Active(GameStateCommand::Play));
		app.update();

		assert_eq!(
			(
				&CommandState::active(GameStateCommand::Play),
				&CommandState::active(GameStateCommandExtended::Command(GameStateCommand::Play)),
			),
			(
				app.world()
					.resource::<State<CommandState<GameStateCommand>>>()
					.get(),
				app.world()
					.resource::<State<CommandState<GameStateCommandExtended<_A>>>>()
					.get(),
			)
		);
	}

	#[test]
	fn sync_extended_base_state() {
		let mut app = setup();

		app.world_mut()
			.trigger(StateEvent::Active(GameStateCommandExtended::<_A>::Command(
				GameStateCommand::Save,
			)));
		app.update();

		assert_eq!(
			(
				&CommandState::active(GameStateCommand::Save),
				&CommandState::active(GameStateCommandExtended::Command(GameStateCommand::Save)),
			),
			(
				app.world()
					.resource::<State<CommandState<GameStateCommand>>>()
					.get(),
				app.world()
					.resource::<State<CommandState<GameStateCommandExtended<_A>>>>()
					.get(),
			)
		);
	}

	#[test]
	fn sync_extended_extension_state() {
		let mut app = setup();

		app.world_mut()
			.trigger(StateEvent::Active(GameStateCommandExtended::Extended(_A)));
		app.update();

		assert_eq!(
			(
				&CommandState::dirty(),
				&CommandState::active(GameStateCommandExtended::Extended(_A)),
			),
			(
				app.world()
					.resource::<State<CommandState<GameStateCommand>>>()
					.get(),
				app.world()
					.resource::<State<CommandState<GameStateCommandExtended<_A>>>>()
					.get(),
			)
		);
	}

	#[test]
	fn sync_two_extended_states() {
		let mut app = setup();

		app.world_mut()
			.trigger(StateEvent::Active(GameStateCommandExtended::Extended(_A)));
		app.update();

		assert_eq!(
			(
				&CommandState::active(GameStateCommandExtended::Extended(_A)),
				&CommandState::dirty(),
			),
			(
				app.world()
					.resource::<State<CommandState<GameStateCommandExtended<_A>>>>()
					.get(),
				app.world()
					.resource::<State<CommandState<GameStateCommandExtended<_B>>>>()
					.get(),
			)
		);
	}

	#[test]
	fn sync_two_extended_states_reversed() {
		let mut app = setup();

		app.world_mut()
			.trigger(StateEvent::Active(GameStateCommandExtended::Extended(_B)));
		app.update();

		assert_eq!(
			(
				&CommandState::dirty(),
				&CommandState::active(GameStateCommandExtended::Extended(_B)),
			),
			(
				app.world()
					.resource::<State<CommandState<GameStateCommandExtended<_A>>>>()
					.get(),
				app.world()
					.resource::<State<CommandState<GameStateCommandExtended<_B>>>>()
					.get(),
			)
		);
	}
}
