mod resources;
mod states;
mod system_params;

use crate::{
	states::GameState,
	system_params::{game_states_read::GameStatesRead, game_states_write::GameStatesWrite},
};
use bevy::{ecs::system::ScheduleSystem, prelude::*};
use common::traits::{
	handles_game_states::{HandlesGameState, OnState},
	thread_safe::ThreadSafe,
};

pub struct GameStatesPlugin;

impl Plugin for GameStatesPlugin {
	fn build(&self, app: &mut App) {
		app.init_state::<GameState>();
	}
}

impl<T> HandlesGameState<T> for GameStatesPlugin
where
	T: ThreadSafe + Into<GameState>,
{
	type TGameStates = GameStatesRead<'static, T>;
	type TGameStatesMut = GameStatesWrite<'static, T>;

	fn add_systems<M>(
		app: &mut App,
		on_state: OnState<T>,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	) {
		match on_state {
			OnState::Enter(state) => {
				app.add_systems(OnEnter(state.into()), systems);
			}
			OnState::Exit(state) => {
				app.add_systems(OnExit(state.into()), systems);
			}
		}
	}
}
