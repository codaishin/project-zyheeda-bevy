use crate::{events::StateEvent, states::state_internal::StateInternal};
use bevy::prelude::*;
use common::prelude::*;
use std::{any::TypeId, fmt::Debug, hash::Hash};

impl StateEvent<GameState> {
	pub(crate) fn set_game_state(
		on_state: On<StateEvent<GameState>>,
		mut next_state: ResMut<NextState<StateInternal<GameState>>>,
	) {
		next_state.set((*on_state).into());
	}
}

impl<T> StateEvent<GameStateExtended<T>>
where
	T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	pub(crate) fn set_game_state(
		on_state: On<StateEvent<GameState>>,
		mut next_state: ResMut<NextState<StateInternal<GameStateExtended<T>>>>,
	) {
		let next = match *on_state {
			StateEvent::Active(cmd) => StateInternal::active(GameStateExtended::Base(cmd)),
			StateEvent::Dirty { issued_by } if issued_by != Self::id() => StateInternal::dirty(),
			_ => return,
		};

		next_state.set(next);
	}

	pub(crate) fn set_game_state_extension(
		on_state: On<StateEvent<GameStateExtended<T>>>,
		mut commands: ZyheedaCommands,
		mut next_state: ResMut<NextState<StateInternal<GameStateExtended<T>>>>,
	) {
		let base = match *on_state {
			StateEvent::Active(GameStateExtended::Base(cmd)) => StateEvent::Active(cmd),
			_ => StateEvent::Dirty {
				issued_by: Self::id(),
			},
		};

		commands.trigger_observers_for(base);
		next_state.set((*on_state).into());
	}

	fn id() -> TypeId {
		TypeId::of::<GameStateExtended<T>>()
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
		app.init_state::<StateInternal<GameState>>();
		app.add_observer(StateEvent::<GameState>::set_game_state);

		app.init_state::<StateInternal<GameStateExtended<_A>>>();
		app.add_observer(StateEvent::<GameStateExtended<_A>>::set_game_state);
		app.add_observer(StateEvent::<GameStateExtended<_A>>::set_game_state_extension);

		app.init_state::<StateInternal<GameStateExtended<_B>>>();
		app.add_observer(StateEvent::<GameStateExtended<_B>>::set_game_state);
		app.add_observer(StateEvent::<GameStateExtended<_B>>::set_game_state_extension);

		app
	}

	#[test]
	fn sync_base_state() {
		let mut app = setup();

		app.world_mut().trigger(StateEvent::Active(GameState::Play));
		app.update();

		assert_eq!(
			(
				&StateInternal::active(GameState::Play),
				&StateInternal::active(GameStateExtended::Base(GameState::Play)),
			),
			(
				app.world()
					.resource::<State<StateInternal<GameState>>>()
					.get(),
				app.world()
					.resource::<State<StateInternal<GameStateExtended<_A>>>>()
					.get(),
			)
		);
	}

	#[test]
	fn sync_extended_base_state() {
		let mut app = setup();

		app.world_mut()
			.trigger(StateEvent::Active(GameStateExtended::<_A>::Base(
				GameState::Save,
			)));
		app.update();

		assert_eq!(
			(
				&StateInternal::active(GameState::Save),
				&StateInternal::active(GameStateExtended::Base(GameState::Save)),
			),
			(
				app.world()
					.resource::<State<StateInternal<GameState>>>()
					.get(),
				app.world()
					.resource::<State<StateInternal<GameStateExtended<_A>>>>()
					.get(),
			)
		);
	}

	#[test]
	fn sync_extended_extension_state() {
		let mut app = setup();

		app.world_mut()
			.trigger(StateEvent::Active(GameStateExtended::Extended(_A)));
		app.update();

		assert_eq!(
			(
				&StateInternal::dirty(),
				&StateInternal::active(GameStateExtended::Extended(_A)),
			),
			(
				app.world()
					.resource::<State<StateInternal<GameState>>>()
					.get(),
				app.world()
					.resource::<State<StateInternal<GameStateExtended<_A>>>>()
					.get(),
			)
		);
	}

	#[test]
	fn sync_two_extended_states() {
		let mut app = setup();

		app.world_mut()
			.trigger(StateEvent::Active(GameStateExtended::Extended(_A)));
		app.update();

		assert_eq!(
			(
				&StateInternal::active(GameStateExtended::Extended(_A)),
				&StateInternal::dirty(),
			),
			(
				app.world()
					.resource::<State<StateInternal<GameStateExtended<_A>>>>()
					.get(),
				app.world()
					.resource::<State<StateInternal<GameStateExtended<_B>>>>()
					.get(),
			)
		);
	}

	#[test]
	fn sync_two_extended_states_reversed() {
		let mut app = setup();

		app.world_mut()
			.trigger(StateEvent::Active(GameStateExtended::Extended(_B)));
		app.update();

		assert_eq!(
			(
				&StateInternal::dirty(),
				&StateInternal::active(GameStateExtended::Extended(_B)),
			),
			(
				app.world()
					.resource::<State<StateInternal<GameStateExtended<_A>>>>()
					.get(),
				app.world()
					.resource::<State<StateInternal<GameStateExtended<_B>>>>()
					.get(),
			)
		);
	}
}
