use bevy::{
	ecs::system::{ScheduleSystem, SystemParam},
	prelude::*,
};
use std::collections::HashSet;

use crate::traits::thread_safe::ThreadSafe;

pub trait HandlesGameStates:
	HandlesGameState<ActivityState> + HandlesGameState<SaveState> + HandlesGameState<MenuState>
{
}

impl<T> HandlesGameStates for T where
	T: HandlesGameState<ActivityState> + HandlesGameState<SaveState> + HandlesGameState<MenuState>
{
}

pub trait HandlesGameState<T>
where
	T: ThreadSafe,
{
	type TGameStates: SystemParam + GameStates<T>;
	type TGameStatesMut: SystemParam + GameStatesMut<T>;

	fn add_systems<M>(
		app: &mut App,
		on_state: OnState<T>,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	);
}

pub trait GameStates<T>
where
	T: ThreadSafe,
{
	fn game_states(&self) -> &HashSet<T>;
}

pub trait GameStatesMut<T>: GameStates<T>
where
	T: ThreadSafe,
{
	fn game_states_mut(&mut self) -> &mut HashSet<T>;
}

#[derive(Debug, PartialEq)]
pub enum OnState<T> {
	Enter(T),
	Exit(T),
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
