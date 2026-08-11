mod resources;
mod states;
mod system_params;
mod systems;

use crate::{
	resources::game_state_context::GameStatesContext,
	states::activity::Activity,
	system_params::{
		game_states_read::GameStatesRead,
		game_states_write::GameStatesWrite,
		ui_states::UIStates,
	},
};
use bevy::{ecs::system::ScheduleSystem, prelude::*};
use common::{
	tools::plugin_system_set::PluginSystemSet,
	traits::{
		handles_game_states::{GameState, HandlesGameStates, OnGameState},
		system_set_definition::SystemSetDefinition,
	},
};

pub struct GameStatesPlugin;

impl Plugin for GameStatesPlugin {
	fn build(&self, app: &mut App) {
		UIStates::init(app);

		app.init_resource::<GameStatesContext>()
			.init_state::<Activity>()
			.add_systems(
				Update,
				GameStatesContext::sync_states.in_set(GameStateSystems),
			);
	}
}

#[derive(SystemSet, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub struct GameStateSystems;

impl HandlesGameStates for GameStatesPlugin {
	type TGameStates = GameStatesRead<'static>;
	type TGameStatesMut = GameStatesWrite<'static>;

	fn add_systems<M>(
		app: &mut App,
		on_state: OnGameState,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	) {
		match on_state {
			OnGameState::Enter(GameState::Activity(activity)) => {
				app.add_systems(OnEnter(Activity::from(activity)), systems);
			}
			OnGameState::Exit(GameState::Activity(activity)) => {
				app.add_systems(OnExit(Activity::from(activity)), systems);
			}
			OnGameState::Enter(GameState::Read(read)) => {
				app.add_systems(OnEnter(Activity::from(read)), systems);
			}
			OnGameState::Exit(GameState::Read(read)) => {
				app.add_systems(OnExit(Activity::from(read)), systems);
			}
			OnGameState::Enter(GameState::IngameUI(ui)) => {
				UIStates::on_enter(app, ui, systems);
			}
			OnGameState::Exit(GameState::IngameUI(ui)) => {
				UIStates::on_exit(app, ui, systems);
			}
		}
	}
}

impl SystemSetDefinition for GameStatesPlugin {
	type TSystemSet = GameStateSystems;

	const SYSTEMS: PluginSystemSet<Self::TSystemSet> = PluginSystemSet::from_set(GameStateSystems);
}
