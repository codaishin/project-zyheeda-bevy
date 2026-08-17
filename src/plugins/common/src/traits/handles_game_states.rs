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
	AddGameStateSystem + AutomaticActivityTransitions + NonPausedStates
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

pub trait AutomaticActivityTransitions {
	type TOptionalTransitions<'a>: WithOptionalActivityTransitions;

	fn automatic_game_state_transitions(
		app: &mut App,
		from_state: Activity,
		to_state: ActivityTransition,
	) -> Result<Self::TOptionalTransitions<'_>, TransitionsConfigError>;
}

pub trait WithOptionalActivityTransitions {
	fn with_optional_transitions<TResult, M>(
		self,
		check: impl IntoSystem<(), Option<TResult>, M>,
		transitions: HashMap<TResult, ActivityTransition>,
	) -> Result<(), TransitionsConfigError>
	where
		TResult: PartialEq + Eq + Hash + ThreadSafe;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ActivityTransition {
	To(Activity),
	ToPrevious,
}

#[derive(Debug, PartialEq)]
pub enum TransitionsConfigError {
	AlreadyConfigured(Activity),
	MayNotTransitionToSelf(Activity),
}

impl Display for TransitionsConfigError {
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
	const DEFAULT: &[Activity] = &[Activity::Settable(SettableActivity::Play)];

	fn add_non_pause_state(app: &mut App, state: impl Into<GameState>);
}

pub trait GameStates {
	fn activity(&self) -> Activity;
	fn ui(&self) -> &'_ HashSet<IngameUI>;
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
	activity: Option<Activity>,
	ui: HashSetIter<'a, IngameUI>,
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
	fn set_activity(&mut self, activity: SettableActivity);
	fn ui_mut(&mut self) -> &'_ mut HashSet<IngameUI>;
}

#[derive(Debug, PartialEq)]
pub enum OnGameState {
	Enter(GameState),
	Exit(GameState),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum GameState {
	Activity(Activity),
	IngameUI(IngameUI),
}

impl_enum_conversions!(GameState[
	Activity(Activity),
	IngameUI(IngameUI),
]);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Activity {
	Settable(SettableActivity),
	Derived(DerivedActivity),
}

impl_enum_conversions!(Activity[
	Settable(SettableActivity),
	Derived(DerivedActivity),
]);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SettableActivity {
	StartScreen,
	NewGame,
	Play,
	Paused,
	Save,
	Load,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum DerivedActivity {
	LoadingEssentialAssets,
	LoadDependencies,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum IngameUI {
	Hud,
	Inventory,
	ComboOverview,
	Settings,
}

impl IterFinite for IngameUI {
	fn iterator() -> FiniteIter<Self> {
		FiniteIter(Some(IngameUI::Hud))
	}

	fn next(current: &FiniteIter<Self>) -> Option<Self> {
		match current.0? {
			IngameUI::Hud => Some(IngameUI::Inventory),
			IngameUI::Inventory => Some(IngameUI::ComboOverview),
			IngameUI::ComboOverview => Some(IngameUI::Settings),
			IngameUI::Settings => None,
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
				IngameUI::Hud,
				IngameUI::Inventory,
				IngameUI::ComboOverview,
				IngameUI::Settings,
			],
			IngameUI::iterator().take(100).collect::<Vec<_>>()
		);
	}

	#[test]
	fn iter_game_state_collection() {
		struct _GameStates {
			activity: Activity,
			ui: HashSet<IngameUI>,
		}

		impl GameStates for &'_ _GameStates {
			fn activity(&self) -> Activity {
				self.activity
			}

			fn ui(&self) -> &'_ HashSet<IngameUI> {
				&self.ui
			}
		}

		let game_states = &_GameStates {
			activity: Activity::Settable(SettableActivity::Play),
			ui: HashSet::from([IngameUI::Hud, IngameUI::Settings]),
		};

		assert_eq!(
			HashSet::from([
				GameState::Activity(Activity::Settable(SettableActivity::Play)),
				GameState::IngameUI(IngameUI::Hud),
				GameState::IngameUI(IngameUI::Settings)
			]),
			game_states.iter().collect::<HashSet<_>>()
		);
	}
}
