use bevy::{prelude::*, state::state::FreelyMutableState};

pub mod game_state;
pub mod menu_state;
pub mod mouse_context;

pub fn transition_to_state<TState: FreelyMutableState + Copy>(
	state: TState,
) -> impl Fn(ResMut<NextState<TState>>) {
	move |mut next_state: ResMut<NextState<TState>>| {
		next_state.set(state);
	}
}
