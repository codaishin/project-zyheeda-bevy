use bevy::{
	ecs::system::{ScheduleSystem, SystemParam},
	prelude::*,
};
use std::collections::HashSet;

pub trait HandlesGameStates {
	type TGameStates: SystemParam + GameStates;
	type TGameStatesMut: SystemParam + GameStatesMut;

	fn add_systems<M>(
		app: &mut App,
		on_state: OnGameState,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	);
}

pub trait GameStatesMut: GameStates + AddGameState + RemoveGameState {}

impl<T> GameStatesMut for T where T: GameStates + AddGameState + RemoveGameState {}

pub trait IntoGameStateAdd: Into<GameState> + game_state::CanAdd {}
pub trait IntoGameStateRemove: Into<GameState> + game_state::CanRemove {}

pub trait GameStates {
	fn game_states(&self) -> &HashSet<GameState>;
}

pub trait AddGameState {
	fn add_game_state<T>(&mut self, state: T)
	where
		T: IntoGameStateAdd;
}

pub trait RemoveGameState {
	fn remove_game_state<T>(&mut self, state: &T)
	where
		T: IntoGameStateRemove + Copy;
}

#[derive(Debug, PartialEq)]
pub enum OnGameState {
	Enter(GameState),
	Exit(GameState),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum GameState {
	Activity(ActivityState),
	IngameUI(UIState),
	Read(ReadState),
}

macro_rules! impl_into_game_state {
	($wrapper:ident($ty:ty)) => {
		impl From<$ty> for GameState {
			fn from(value: $ty) -> Self {
				Self::$wrapper(value)
			}
		}
	};
}

impl_into_game_state!(Activity(ActivityState));
impl_into_game_state!(IngameUI(UIState));
impl_into_game_state!(Read(ReadState));

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ActivityState {
	LoadingEssentialAssets,
	LoadDependencies,
	StartScreen,
	NewGame,
	Play,
	Paused,
	Save,
	Load,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum UIState {
	Hud,
	Inventory,
	ComboOverview,
	Settings,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ReadState {
	Loading,
}

mod game_state {
	use super::*;

	pub trait CanAdd {}

	impl<T> IntoGameStateAdd for T where T: Into<GameState> + CanAdd {}

	impl CanAdd for ActivityState {}
	impl CanAdd for UIState {}

	pub trait CanRemove {}

	impl<T> IntoGameStateRemove for T where T: Into<GameState> + CanRemove {}

	impl CanRemove for UIState {}
}
