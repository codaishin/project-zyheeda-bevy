mod resources;
mod states;
mod system_params;

use crate::{
	resources::game_state_context::GameStatesContext,
	states::GameStateInternal,
	system_params::{game_states_read::GameStatesRead, game_states_write::GameStatesWrite},
};
use bevy::{ecs::system::ScheduleSystem, prelude::*};
use common::traits::handles_game_states::{HandlesGameStates, OnGameState};

pub struct GameStatesPlugin;

impl Plugin for GameStatesPlugin {
	fn build(&self, app: &mut App) {
		app.init_resource::<GameStatesContext>()
			.init_state::<GameStateInternal>();
	}
}

impl HandlesGameStates for GameStatesPlugin {
	type TGameStates = GameStatesRead<'static>;
	type TGameStatesMut = GameStatesWrite<'static>;

	fn add_systems<M>(
		app: &mut App,
		on_state: OnGameState,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	) {
		match on_state {
			OnGameState::Enter(state) => {
				app.add_systems(OnEnter(GameStateInternal(state)), systems);
			}
			OnGameState::Exit(state) => {
				app.add_systems(OnExit(GameStateInternal(state)), systems);
			}
		}
	}
}
