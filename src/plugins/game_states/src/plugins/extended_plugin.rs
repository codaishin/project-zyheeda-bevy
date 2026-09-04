use bevy::{prelude::*, state::state::StateTransitionSystems};
use common::prelude::*;
use std::{fmt::Debug, hash::Hash, marker::PhantomData};

use crate::{
	GameStateSystems,
	GameStatesPlugin,
	PreviousStates,
	events::StateEvent,
	states::state_internal::StateInternal,
};

pub(crate) struct ExtendedPlugin<TExtended>(PhantomData<TExtended>);

impl<TExtended> Default for ExtendedPlugin<TExtended> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

impl<TExtended> Plugin for ExtendedPlugin<TExtended>
where
	TExtended: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy,
{
	fn build(&self, app: &mut App) {
		let set_game_state = StateEvent::<GameStateExtended<TExtended>>::set_game_state;
		let set_game_state_extension =
			StateEvent::<GameStateExtended<TExtended>>::set_game_state_extension;

		app.init_state::<StateInternal<GameStateExtended<TExtended>>>()
			.init_resource::<PreviousStates<GameStateExtended<TExtended>>>()
			.add_observer(set_game_state)
			.add_observer(set_game_state_extension)
			.add_systems(
				StateTransition,
				GameStatesPlugin::track_transitions::<GameStateExtended<TExtended>>
					.in_set(GameStateSystems)
					.after(StateTransitionSystems::EnterSchedules),
			);
	}
}
