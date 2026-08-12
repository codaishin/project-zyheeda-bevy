use crate::{
	tools::is_not::IsNot,
	traits::iteration::{Iter, IterFinite},
};
use bevy::{
	ecs::system::{ScheduleSystem, SystemParam},
	prelude::*,
};
use std::{
	collections::{HashMap, HashSet},
	fmt::{Debug, Display},
	hash::Hash,
};

pub trait HandlesGameStates:
	AddGameStateSystem + AutomaticGameStateTransitions<ActivityState>
{
	type TGameStates: SystemParam + GameStates;
	type TGameStatesMut: SystemParam + GameStatesMut;
}

pub trait AddGameStateSystem {
	fn add_game_state_systems<M>(
		app: &mut App,
		on_state: OnGameState,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	);
}

pub trait AutomaticGameStateTransitions<T> {
	type TOptionalTransitions<'a>: WithOptionalTransitions<T>;

	fn automatic_game_state_transitions(
		app: &mut App,
		from_state: T,
		to_state: StateTransition<T>,
	) -> Result<Self::TOptionalTransitions<'_>, TransitionsConfigError<T>>;
}

pub trait WithOptionalTransitions<T> {
	fn with_optional_transitions<TResult, M>(
		self,
		check: impl IntoSystem<(), Option<TResult>, M>,
		transitions: HashMap<TResult, StateTransition<T>>,
	) -> Result<(), TransitionsConfigError<T>>
	where
		TResult: PartialEq + Eq + Hash;
}

pub enum StateTransition<T> {
	To(T),
	ToPrevious,
}

#[derive(Debug, PartialEq)]
pub enum TransitionsConfigError<TState> {
	AlreadyConfigured(TState),
	MayNotTransitionToSelf(TState),
}

impl<TState> Display for TransitionsConfigError<TState>
where
	TState: Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			TransitionsConfigError::AlreadyConfigured(state) => {
				write!(f, "{state:?}: Cannot be configure more than once")
			}
			TransitionsConfigError::MayNotTransitionToSelf(state) => {
				write!(f, "{state:?}: May automatically transition back to itself")
			}
		}
	}
}

pub trait GameStatesMut:
	GameStates + AddGameState<ActivityState> + AddGameState<UIState> + RemoveGameState<UIState>
{
}

impl<T> GameStatesMut for T where
	T: GameStates + AddGameState<ActivityState> + AddGameState<UIState> + RemoveGameState<UIState>
{
}

pub trait GameStates {
	fn game_states(&self) -> &HashSet<GameState>;
}

pub trait AddGameState<T> {
	fn add_game_state(&mut self, state: T);
}

pub trait RemoveGameState<T> {
	fn remove_game_state(&mut self, state: &T);
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

macro_rules! game_state_conversions {
	($wrapper:ident($inner:ty)) => {
		impl From<$inner> for GameState {
			fn from(value: $inner) -> Self {
				Self::$wrapper(value)
			}
		}

		impl TryFrom<GameState> for $inner {
			type Error = IsNot<$inner>;

			fn try_from(game_state: GameState) -> Result<$inner, Self::Error> {
				match game_state {
					GameState::$wrapper(inner) => Ok(inner),
					_ => Err(IsNot::target_type()),
				}
			}
		}
	};
}

game_state_conversions!(Activity(ActivityState));
game_state_conversions!(IngameUI(UIState));
game_state_conversions!(Read(ReadState));

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

impl IterFinite for UIState {
	fn iterator() -> Iter<Self> {
		Iter(Some(UIState::Hud))
	}

	fn next(current: &Iter<Self>) -> Option<Self> {
		match current.0? {
			UIState::Hud => Some(UIState::Inventory),
			UIState::Inventory => Some(UIState::ComboOverview),
			UIState::ComboOverview => Some(UIState::Settings),
			UIState::Settings => None,
		}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ReadState {
	Loading,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_ui_states() {
		assert_eq!(
			vec![
				UIState::Hud,
				UIState::Inventory,
				UIState::ComboOverview,
				UIState::Settings,
			],
			UIState::iterator().take(100).collect::<Vec<_>>()
		);
	}
}
