pub mod game_state;
pub mod menu_state;
pub mod save_state;

use bevy::{prelude::*, state::state::FreelyMutableState};

pub fn transition_to_state<TState>(state: TState) -> impl Fn(ResMut<NextState<TState>>)
where
	TState: FreelyMutableState + Clone,
{
	move |mut next_state: ResMut<NextState<TState>>| {
		next_state.set(state.clone());
	}
}

pub fn transition_to_previous<TState>(
	mut transition_messages: MessageReader<StateTransitionEvent<TState>>,
	mut next_state: ResMut<NextState<TState>>,
) where
	TState: FreelyMutableState + Clone,
{
	let Some(last_transition) = transition_messages.read().last() else {
		return;
	};
	let Some(previous) = last_transition.exited.as_ref() else {
		return;
	};

	next_state.set(previous.clone());
}
