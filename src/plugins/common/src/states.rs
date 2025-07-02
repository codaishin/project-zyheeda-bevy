pub mod game_state;
pub mod menu_state;

use bevy::{prelude::*, state::state::FreelyMutableState};

pub fn transition_to_state<TState>(state: TState) -> impl Fn(ResMut<NextState<TState>>)
where
	TState: FreelyMutableState + Clone,
{
	move |mut next_state: ResMut<NextState<TState>>| {
		next_state.set(state.clone());
	}
}
