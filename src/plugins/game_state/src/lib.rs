mod resources;
mod systems;

use crate::resources::last_state::LastState;
use bevy::{ecs::system::ScheduleSystem, prelude::*};
use common::{
	states::{
		game_state::{GameState, LoadingEssentialAssets, LoadingGame},
		transition_to_state,
	},
	traits::{
		handles_game_states::HandlesGameStates,
		handles_load_tracking::{HandlesLoadTracking, LoadGroup},
		thread_safe::ThreadSafe,
	},
};
use std::marker::PhantomData;
use systems::pause_virtual_time::pause_virtual_time;

pub struct GameStatePlugin<TDependencies>(PhantomData<TDependencies>);

impl<TLoading> GameStatePlugin<TLoading>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
{
	pub fn from_plugin(_: &TLoading) -> Self {
		GameStatePlugin(PhantomData)
	}
}

impl<TLoading> Plugin for GameStatePlugin<TLoading>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
{
	fn build(&self, app: &mut App) {
		let loading_essentials = LoadingEssentialAssets::LOAD_STATE;
		let new_game = GameState::NewGame;
		let loading = LoadingGame::LOAD_STATE;
		let saving = GameState::Saving;
		let loading_save = GameState::LoadingSave;
		let play = GameState::Play;

		app.insert_state(loading_essentials)
			.init_resource::<LastState<GameState>>()
			.add_systems(StateTransition, LastState::<GameState>::track)
			.add_systems(OnEnter(new_game), transition_to_state(loading))
			.add_systems(OnEnter(saving), LastState::<GameState>::transition)
			.add_systems(OnEnter(loading_save), transition_to_state(loading))
			.add_systems(OnEnter(play), pause_virtual_time::<false>)
			.add_systems(OnExit(play), pause_virtual_time::<true>);
	}
}

impl<TLoading> HandlesGameStates for GameStatePlugin<TLoading> {
	fn on_starting_new_game<TSystem, TMarker>(app: &mut App, systems: TSystem)
	where
		TSystem: IntoScheduleConfigs<ScheduleSystem, TMarker>,
	{
		app.add_systems(OnEnter(GameState::NewGame), systems);
	}
}
