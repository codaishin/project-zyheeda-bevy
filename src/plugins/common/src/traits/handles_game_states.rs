use crate::traits::{
	iteration::{FiniteIter, IterFinite},
	thread_safe::ThreadSafe,
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
use zyheeda_core::prelude::*;

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
	const DEFAULT: &[ActivityState] = &[ActivityState::Settable(SettableState::Play)];

	fn add_non_pause_state(app: &mut App, state: impl Into<GameState>);
}

pub trait GameStates {
	fn activity(&self) -> ActivityState;
	fn ui(&self) -> &'_ HashSet<UIState>;
}

pub trait IterGameStates: GameStates {
	fn iter(&self) -> GameStateIter<'_> {
		GameStateIter {
			activity: Some(self.activity()),
			ui: self.ui().iter(),
		}
	}
}

impl<T> IterGameStates for T where T: GameStates {}

pub struct GameStateIter<'a> {
	activity: Option<ActivityState>,
	ui: HashSetIter<'a, UIState>,
}

impl Iterator for GameStateIter<'_> {
	type Item = GameState;

	fn next(&mut self) -> Option<Self::Item> {
		match self.activity.take() {
			Some(activity) => Some(GameState::Activity(activity)),
			None => self.ui.next().copied().map(GameState::IngameUI),
		}
	}
}

pub trait GameStatesMut {
	fn set_activity(&mut self, activity: SettableState);
	fn ui_mut(&mut self) -> &'_ mut HashSet<UIState>;
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

impl_enum_conversions!(GameState[
	Activity(ActivityState),
	IngameUI(UIState),
]);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ActivityState {
	Settable(SettableState),
	Derived(DerivedState),
}

impl_enum_conversions!(ActivityState[
	Settable(SettableState),
	Derived(DerivedState),
]);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SettableState {
	StartScreen,
	NewGame,
	Play,
	Paused,
	Save,
	Load,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum DerivedState {
	LoadingEssentialAssets,
	LoadDependencies,
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
		struct _GameStates {
			activity: ActivityState,
			ui: HashSet<UIState>,
		}

		impl GameStates for &'_ _GameStates {
			fn activity(&self) -> ActivityState {
				self.activity
			}

			fn ui(&self) -> &'_ HashSet<UIState> {
				&self.ui
			}
		}

		let game_states = &_GameStates {
			activity: ActivityState::Settable(SettableState::Play),
			ui: HashSet::from([UIState::Hud, UIState::Settings]),
		};

		assert_eq!(
			HashSet::from([
				GameState::Activity(ActivityState::Settable(SettableState::Play)),
				GameState::IngameUI(UIState::Hud),
				GameState::IngameUI(UIState::Settings)
			]),
			game_states.iter().collect::<HashSet<_>>()
		);
	}
}
