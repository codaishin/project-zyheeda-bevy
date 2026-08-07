use bevy::{
	ecs::system::{ScheduleSystem, SystemParam},
	prelude::*,
};
use zyheeda_core::collections::ordered::OrderedSet;

pub trait HandlesGameStates {
	type TGameStates: SystemParam + GameStates;
	type TGameStatesMut: SystemParam + GameStatesMut;

	fn add_systems<M>(
		app: &mut App,
		on_state: OnGameState,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	);
}

pub trait GameStates {
	fn game_states(&self) -> &OrderedSet<GameState>;
}

pub trait GameStatesMut: GameStates {
	fn game_states_mut(&mut self) -> &mut OrderedSet<GameState>;
}

#[derive(Debug, PartialEq)]
pub enum OnGameState {
	Enter(GameState),
	Exit(GameState),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum GameState {
	Activity(ActivityState),
	Save(SaveState),
	Menu(MenuState),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ActivityState {
	LoadingEssentialAssets,
	LoadDependencies,
	NewGame,
	Play,
	Paused,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SaveState {
	Save,
	LoadAttempt,
	Load,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum MenuState {
	StartMenu,
	Inventory,
	ComboOverview,
	Settings,
}
