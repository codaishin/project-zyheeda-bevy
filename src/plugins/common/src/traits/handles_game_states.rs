use crate::{
	tools::is_not::IsNot,
	traits::{
		iteration::{Iter as FiniteIter, IterFinite},
		thread_safe::ThreadSafe,
	},
};
use bevy::{
	ecs::system::{ScheduleSystem, SystemParam},
	prelude::*,
};
use std::{
	collections::{HashMap, HashSet, hash_set::Iter as HashSetIter},
	fmt::{Debug, Display},
	hash::Hash,
};

pub trait HandlesGameStates:
	AddGameStateSystem + AutomaticGameStateTransitions<ActivityState> + NonPausedStates
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
		to_state: GameStateTransition<T>,
	) -> Result<Self::TOptionalTransitions<'_>, TransitionsConfigError<T>>;
}

pub trait WithOptionalTransitions<T> {
	fn with_optional_transitions<TResult, M>(
		self,
		check: impl IntoSystem<(), Option<TResult>, M>,
		transitions: HashMap<TResult, GameStateTransition<T>>,
	) -> Result<(), TransitionsConfigError<T>>
	where
		TResult: PartialEq + Eq + Hash + ThreadSafe;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum GameStateTransition<T> {
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
				write!(f, "{state:?}: Cannot be configured more than once")
			}
			TransitionsConfigError::MayNotTransitionToSelf(state) => {
				write!(
					f,
					"{state:?}: May not automatically transition back to itself"
				)
			}
		}
	}
}

pub trait NonPausedStates {
	const DEFAULT: &[ActivityState] = &[ActivityState::Play];

	fn add_non_pause_state(app: &mut App, state: impl Into<GameState>);
}

pub trait GameStates {
	fn game_states(&self) -> GameStateCollection<'_>;
}

pub trait GameStatesMut: GameStates {
	fn game_states_mut(&mut self) -> GameStateCollectionMut<'_>;
}

#[derive(Debug, PartialEq, Clone)]
pub struct GameStateCollection<'a> {
	pub activity: ActivityState,
	pub ui: &'a HashSet<UIState>,
}

impl GameStateCollection<'_> {
	pub fn iter(&self) -> Iter<'_> {
		Iter {
			activity: Some(&self.activity),
			ui: self.ui.iter(),
		}
	}
}

pub struct Iter<'a> {
	activity: Option<&'a ActivityState>,
	ui: HashSetIter<'a, UIState>,
}

impl Iterator for Iter<'_> {
	type Item = GameState;

	fn next(&mut self) -> Option<Self::Item> {
		match self.activity.take().copied() {
			Some(activity) => Some(GameState::Activity(activity)),
			None => self.ui.next().copied().map(GameState::IngameUI),
		}
	}
}

#[derive(Debug, PartialEq)]
pub struct GameStateCollectionMut<'a> {
	pub current: GameStateCollection<'a>,
	pub next: &'a mut NextGameStates,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NextGameStates {
	pub activity: ActivityState,
	pub ui: HashSet<UIState>,
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
	fn iterator() -> FiniteIter<Self> {
		FiniteIter(Some(UIState::Hud))
	}

	fn next(current: &FiniteIter<Self>) -> Option<Self> {
		match current.0? {
			UIState::Hud => Some(UIState::Inventory),
			UIState::Inventory => Some(UIState::ComboOverview),
			UIState::ComboOverview => Some(UIState::Settings),
			UIState::Settings => None,
		}
	}
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

	#[test]
	fn iter_game_state_collection() {
		let game_states = GameStateCollection {
			activity: ActivityState::Play,
			ui: &HashSet::from([UIState::Hud, UIState::Settings]),
		};

		assert_eq!(
			HashSet::from([
				GameState::Activity(ActivityState::Play),
				GameState::IngameUI(UIState::Hud),
				GameState::IngameUI(UIState::Settings)
			]),
			game_states.iter().collect::<HashSet<_>>()
		);
	}
}
