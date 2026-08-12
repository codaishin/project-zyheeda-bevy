mod resources;
mod states;
mod system_params;
mod systems;

use std::collections::HashMap;

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
		handles_game_states::{
			ActivityState,
			AddGameStateSystem,
			AutomaticGameStateTransitions,
			GameState,
			HandlesGameStates,
			OnGameState,
			StateTransition,
			TransitionsConfigError,
			WithOptionalTransitions,
		},
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
}

impl AddGameStateSystem for GameStatesPlugin {
	fn add_game_state_systems<M>(
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

impl AutomaticGameStateTransitions<ActivityState> for GameStatesPlugin {
	type TOptionalTransitions<'a> = OptionalTransitions<'a>;

	fn automatic_game_state_transitions(
		app: &mut App,
		_: ActivityState,
		_: StateTransition<ActivityState>,
	) -> Result<Self::TOptionalTransitions<'_>, TransitionsConfigError<ActivityState>> {
		Ok(OptionalTransitions(app))
	}
}

pub struct OptionalTransitions<'a>(&'a mut App);

impl WithOptionalTransitions<ActivityState> for OptionalTransitions<'_> {
	fn with_optional_transitions<TResult, M>(
		self,
		_: impl IntoSystem<(), Option<TResult>, M>,
		_: HashMap<TResult, StateTransition<ActivityState>>,
	) -> Result<(), TransitionsConfigError<ActivityState>>
	where
		TResult: PartialEq + Eq + std::hash::Hash,
	{
		Ok(())
	}
}

impl SystemSetDefinition for GameStatesPlugin {
	type TSystemSet = GameStateSystems;

	const SYSTEMS: PluginSystemSet<Self::TSystemSet> = PluginSystemSet::from_set(GameStateSystems);
}
