use crate::traits::{
	iteration::{FiniteIter, IterFinite},
	thread_safe::ThreadSafe,
};
use bevy::{
	ecs::system::{ScheduleSystem, SystemParam},
	prelude::*,
};
use macros::EnumConversions;
use std::{
	collections::{HashMap, HashSet},
	fmt::{Debug, Display},
	hash::Hash,
	ops::DerefMut,
};

pub trait HandlesGameStates:
	InGameState<GameState>
	+ InGameState<Gui>
	+ GameStatesWrite
	+ AddGameStateSystem<GameState>
	+ AddGameStateSystem<Gui>
	+ AddActivityTransitions
	+ ExtendGameState
	+ SetToNotPause<GameState>
	+ SetToNotPause<Gui>
	+ GamePaused
{
}

pub trait GameStatesWrite {
	type TGameStatesMut: ThreadSafe + for<'w, 's> SystemParam<Item<'w, 's>: GameStatesMut>;
}

pub trait AddGameStateSystem<TState> {
	fn add_game_state_systems<M>(
		app: &mut App,
		on_state: OnStateTransition<TState>,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	);
}

pub trait AddActivityTransitions<TState = GameState> {
	fn add_activity_transitions<TResult, M>(
		app: &mut App,
		from_state: impl Into<Option<TState>>,
		check: impl IntoSystem<(), Option<TResult>, M>,
		transitions: impl Into<HashMap<TResult, TransitionState<TState>>>,
	) -> Result<(), TransitionsConfigError<TState>>
	where
		TResult: PartialEq + Eq + Hash + ThreadSafe;
}

pub fn always() -> Option<()> {
	Some(())
}

pub trait ExtendGameState {
	type TExtended<T>: InGameState<GameStateExtended<T>>
		+ AddGameStateSystem<GameStateExtended<T>>
		+ AddActivityTransitions<GameStateExtended<T>>
	where
		T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumConversions)]
pub enum GameStateExtended<T> {
	Base(GameState),
	#[enum_conversions(skip)]
	Extended(T),
}

pub trait InGameState<TState> {
	fn in_game_state<const N: usize>(
		game_states: [TState; N],
	) -> impl IntoSystem<(), bool, (), System: ReadOnlySystem>;
}

pub trait GamePaused {
	fn game_paused() -> impl IntoSystem<(), bool, (), System: ReadOnlySystem>;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TransitionState<T = GameState> {
	To(T),
	ToPrevious,
	ToPreviousOf(T),
}

#[derive(Debug, PartialEq)]
pub enum TransitionsConfigError<T = GameState> {
	AlreadyConfigured(Option<T>),
	MayNotTransitionToSelf(T),
}

impl<T> Display for TransitionsConfigError<T>
where
	T: Debug,
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

pub trait SetToNotPause<TState> {
	fn set_to_not_pause(app: &mut App, state: TState);
}

pub trait GameStatesMut {
	type TGameStateSetter<'a>: SetGameState
	where
		Self: 'a;

	#[must_use]
	fn get_game_state_setter(&mut self, state: GameState) -> Option<Self::TGameStateSetter<'_>>;

	fn gui_mut(&mut self) -> &'_ mut HashSet<Gui>;
}

impl<T> GameStatesMut for T
where
	T: DerefMut<Target: GameStatesMut>,
{
	type TGameStateSetter<'a>
		= <T::Target as GameStatesMut>::TGameStateSetter<'a>
	where
		Self: 'a;

	fn get_game_state_setter(&mut self, state: GameState) -> Option<Self::TGameStateSetter<'_>> {
		self.deref_mut().get_game_state_setter(state)
	}

	fn gui_mut(&mut self) -> &'_ mut HashSet<Gui> {
		self.deref_mut().gui_mut()
	}
}

pub trait SetGameState {
	fn set_game_state(self);
}

#[derive(Debug, PartialEq)]
pub enum OnStateTransition<T> {
	Enter(T),
	Exit(T),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum GameState {
	StartScreen,
	NewGame,
	Play,
	Pause,
	Save,
	Load,
}

impl GameState {
	pub const fn extended_base<T>(game_state: GameState) -> GameStateExtended<T> {
		GameStateExtended::Base(game_state)
	}

	pub const fn extended<T>(extended: T) -> GameStateExtended<T> {
		GameStateExtended::Extended(extended)
	}
}

impl IterFinite for GameState {
	fn iterator() -> FiniteIter<Self> {
		FiniteIter(Some(Self::StartScreen))
	}

	fn next(current: &FiniteIter<Self>) -> Option<Self> {
		match current.0? {
			Self::StartScreen => Some(Self::NewGame),
			Self::NewGame => Some(Self::Play),
			Self::Play => Some(Self::Pause),
			Self::Pause => Some(Self::Save),
			Self::Save => Some(Self::Load),
			Self::Load => None,
		}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum SaveGameExtension<T> {
	SaveGame(T),
	LoadGame(T),
}

impl<T> From<SaveGameExtension<T>> for GameStateExtended<SaveGameExtension<T>> {
	fn from(ext: SaveGameExtension<T>) -> Self {
		Self::Extended(ext)
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum LoadAssetsExtension<T> {
	LoadEssentials(T),
	Load(T),
}

impl<T> From<LoadAssetsExtension<T>> for GameStateExtended<LoadAssetsExtension<T>> {
	fn from(ext: LoadAssetsExtension<T>) -> Self {
		Self::Extended(ext)
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Gui {
	Hud,
	Inventory,
	ComboOverview,
	Settings,
}

impl IterFinite for Gui {
	fn iterator() -> FiniteIter<Self> {
		FiniteIter(Some(Gui::Hud))
	}

	fn next(current: &FiniteIter<Self>) -> Option<Self> {
		match current.0? {
			Gui::Hud => Some(Gui::Inventory),
			Gui::Inventory => Some(Gui::ComboOverview),
			Gui::ComboOverview => Some(Gui::Settings),
			Gui::Settings => None,
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn iter_game_states() {
		assert_eq!(
			vec![
				GameState::StartScreen,
				GameState::NewGame,
				GameState::Play,
				GameState::Pause,
				GameState::Save,
				GameState::Load,
			],
			GameState::iterator().take(100).collect::<Vec<_>>()
		);
	}

	#[test]
	fn iter_ui() {
		assert_eq!(
			vec![Gui::Hud, Gui::Inventory, Gui::ComboOverview, Gui::Settings,],
			Gui::iterator().take(100).collect::<Vec<_>>()
		);
	}

	#[test]
	fn iter_ui_states() {
		assert_eq!(
			vec![Gui::Hud, Gui::Inventory, Gui::ComboOverview, Gui::Settings,],
			Gui::iterator().take(100).collect::<Vec<_>>()
		);
	}
}
