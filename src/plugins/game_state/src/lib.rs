mod systems;

use bevy::prelude::*;
use common::{
	states::{game_state::GameState, transition_to_state},
	traits::{
		handles_game_states::HandlesGameStates,
		handles_load_tracking::{HandlesLoadTracking, OnLoadingDone},
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
	pub fn depends_on(_: &TLoading) -> Self {
		GameStatePlugin(PhantomData)
	}
}

impl<TLoading> Plugin for GameStatePlugin<TLoading>
where
	TLoading: ThreadSafe + HandlesLoadTracking,
{
	fn build(&self, app: &mut App) {
		let start_menu = GameState::StartMenu;
		let new_game = GameState::NewGame;
		let loading = GameState::Loading;
		let play = GameState::Play;

		TLoading::begin_loading_on(app, loading).when_done_set(play);

		app.insert_state(start_menu)
			.add_systems(OnEnter(new_game), transition_to_state(loading))
			.add_systems(OnEnter(play), pause_virtual_time::<false>)
			.add_systems(OnExit(play), pause_virtual_time::<true>);
	}
}

impl<TLoading> HandlesGameStates for GameStatePlugin<TLoading> {
	fn on_starting_new_game<TSystem, TMarker>(app: &mut App, systems: TSystem)
	where
		TSystem: IntoSystemConfigs<TMarker>,
	{
		app.add_systems(OnEnter(GameState::NewGame), systems);
	}
}
