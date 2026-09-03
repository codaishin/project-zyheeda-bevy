use crate::{
	tools::iter_helpers::{first, next},
	traits::{
		iteration::{FiniteIter, IterFinite},
		thread_safe::ThreadSafe,
	},
};
use bevy::{
	ecs::system::{ReadOnlySystemParam, ScheduleSystem, StaticSystemParam, SystemParam},
	prelude::*,
};
use macros::EnumConversions;
use std::{
	collections::{HashMap, HashSet, hash_set::Iter as HashSetIter},
	fmt::{Debug, Display},
	hash::Hash,
	ops::{Deref, DerefMut},
};

pub trait HandlesGameStates:
	GameStatesRead
	+ GameStatesWrite
	+ AddGameStateSystem
	+ AddActivityTransitions
	+ ExtendGameState
	+ SetToNotPause
	+ GamePaused
{
}

pub trait GameStatesRead {
	type TGameStates: ThreadSafe + for<'w, 's> ReadOnlySystemParam<Item<'w, 's>: GameStates>;
}

pub trait GameStatesWrite {
	type TGameStatesMut: ThreadSafe + for<'w, 's> SystemParam<Item<'w, 's>: GameStatesMut>;
}

pub trait AddGameStateSystem<TState = GameState> {
	fn add_game_state_systems<M>(
		app: &mut App,
		on_state: OnGameState<TState>,
		systems: impl IntoScheduleConfigs<ScheduleSystem, M>,
	);
}

pub trait AddActivityTransitions<TCommand = GameStateCommand> {
	fn add_activity_transitions<TResult, M>(
		app: &mut App,
		from_state: impl Into<Option<TCommand>>,
		check: impl IntoSystem<(), Option<TResult>, M>,
		transitions: impl Into<HashMap<TResult, ActivityTransition<TCommand>>>,
	) -> Result<(), TransitionsConfigError<TCommand>>
	where
		TResult: PartialEq + Eq + Hash + ThreadSafe;
}

pub fn always() -> Option<()> {
	Some(())
}

pub trait ExtendGameState {
	type TExtended<T>: InGameState<GameStateCommandExtended<T>>
		+ AddGameStateSystem<GameStateCommandExtended<T>>
		+ AddActivityTransitions<GameStateCommandExtended<T>>
	where
		T: ThreadSafe + Debug + PartialEq + Eq + Hash + Clone + Copy;
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumConversions)]
pub enum GameStateCommandExtended<T> {
	Command(GameStateCommand),
	#[enum_conversions(skip)]
	Extended(T),
}

pub trait InGameState<TState = GameState> {
	fn in_game_state<const N: usize, T>(
		game_states: [T; N],
	) -> impl IntoSystem<(), bool, (), System: ReadOnlySystem>
	where
		T: Into<TState>;
}

impl<TPlugin> InGameState for TPlugin
where
	TPlugin: GameStatesRead,
{
	fn in_game_state<const N: usize, T>(
		game_states: [T; N],
	) -> impl IntoSystem<(), bool, (), System: ReadOnlySystem>
	where
		T: Into<GameState>,
	{
		let game_states = game_states.map(|s| s.into());

		IntoSystem::into_system(move |states: StaticSystemParam<TPlugin::TGameStates>| {
			game_states.iter().any(|game_state| match game_state {
				GameState::Command(command) => states.command().as_ref() == Some(command),
				GameState::IngameUI(ui) => states.ui().contains(ui),
			})
		})
	}
}

pub trait GamePaused {
	fn game_paused() -> impl IntoSystem<(), bool, (), System: ReadOnlySystem>;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ActivityTransition<T = GameStateCommand> {
	To(T),
	ToPrevious,
	ToPreviousOf(T),
}

#[derive(Debug, PartialEq)]
pub enum TransitionsConfigError<T = GameStateCommand> {
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

pub trait SetToNotPause {
	const DEFAULT_NON_PAUSE: &[GameState] = &[GameState::Command(GameStateCommand::Play)];

	fn set_to_not_pause(app: &mut App, state: impl Into<GameState>);
}

pub trait GameStates {
	fn command(&self) -> Option<GameStateCommand>;
	fn ui(&self) -> &'_ HashSet<IngameUI>;
}

impl<T> GameStates for T
where
	T: Deref<Target: GameStates>,
{
	fn command(&self) -> Option<GameStateCommand> {
		self.deref().command()
	}

	fn ui(&self) -> &'_ HashSet<IngameUI> {
		self.deref().ui()
	}
}

pub trait IterGameStates: GameStates {
	fn iter(&self) -> GameStateIter<'_> {
		GameStateIter {
			command: self.command(),
			ui: self.ui().iter(),
		}
	}
}

impl<T> IterGameStates for T where T: GameStates {}

pub struct GameStateIter<'a> {
	command: Option<GameStateCommand>,
	ui: HashSetIter<'a, IngameUI>,
}

impl Iterator for GameStateIter<'_> {
	type Item = GameState;

	fn next(&mut self) -> Option<Self::Item> {
		match self.command.take() {
			Some(command) => Some(GameState::Command(command)),
			None => self.ui.next().copied().map(GameState::IngameUI),
		}
	}
}

pub trait GameStatesMut {
	type TGameStateSetter<'a>: SetGameState
	where
		Self: 'a;

	#[must_use]
	fn get_game_state_setter(
		&mut self,
		command: GameStateCommand,
	) -> Option<Self::TGameStateSetter<'_>>;

	fn ui_mut(&mut self) -> &'_ mut HashSet<IngameUI>;
}

impl<T> GameStatesMut for T
where
	T: DerefMut<Target: GameStatesMut>,
{
	type TGameStateSetter<'a>
		= <T::Target as GameStatesMut>::TGameStateSetter<'a>
	where
		Self: 'a;

	fn get_game_state_setter(
		&mut self,
		command: GameStateCommand,
	) -> Option<Self::TGameStateSetter<'_>> {
		self.deref_mut().get_game_state_setter(command)
	}

	fn ui_mut(&mut self) -> &'_ mut HashSet<IngameUI> {
		self.deref_mut().ui_mut()
	}
}

pub trait SetGameState {
	fn set_game_state(self);
}

#[derive(Debug, PartialEq)]
pub enum OnGameState<T = GameState> {
	Enter(T),
	Exit(T),
}

macro_rules! impl_on_game_state_conversions {
	($fst:ty, $($rest:ty),+ $(,)?) => {
		impl_on_game_state_conversions!($fst);
		impl_on_game_state_conversions!($($rest),+);
	};
	($ty:ty) => {
		impl From<OnGameState<$ty>> for OnGameState {
			fn from(value: OnGameState<$ty>) -> Self {
				match value {
					OnGameState::Enter(s) => OnGameState::Enter(s.into()),
					OnGameState::Exit(s) => OnGameState::Exit(s.into()),
				}
			}
		}
	};
}

impl_on_game_state_conversions!(GameStateCommand, IngameUI);

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, EnumConversions)]
pub enum GameState {
	Command(GameStateCommand),
	IngameUI(IngameUI),
}

impl IterFinite for GameState {
	fn iterator() -> FiniteIter<Self> {
		FiniteIter(first(GameState::Command))
	}

	fn next(current: &FiniteIter<Self>) -> Option<Self> {
		use GameState::*;

		match current.0? {
			Command(command) => next(Command, command).or(first(IngameUI)),
			IngameUI(ingame_ui) => next(IngameUI, ingame_ui),
		}
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum GameStateCommand {
	StartScreen,
	NewGame,
	Play,
	Pause,
	Save,
	Load,
}

impl IterFinite for GameStateCommand {
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

impl<T> From<SaveGameExtension<T>> for GameStateCommandExtended<SaveGameExtension<T>> {
	fn from(ext: SaveGameExtension<T>) -> Self {
		Self::Extended(ext)
	}
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum LoadAssetsExtension<T> {
	LoadEssentials(T),
	Load(T),
}

impl<T> From<LoadAssetsExtension<T>> for GameStateCommandExtended<LoadAssetsExtension<T>> {
	fn from(ext: LoadAssetsExtension<T>) -> Self {
		Self::Extended(ext)
	}
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

	mod enum_variants {
		use super::*;

		#[test]
		fn iter_game_states() {
			assert_eq!(
				vec![
					GameState::Command(GameStateCommand::StartScreen),
					GameState::Command(GameStateCommand::NewGame),
					GameState::Command(GameStateCommand::Play),
					GameState::Command(GameStateCommand::Pause),
					GameState::Command(GameStateCommand::Save),
					GameState::Command(GameStateCommand::Load),
					GameState::IngameUI(IngameUI::Hud),
					GameState::IngameUI(IngameUI::Inventory),
					GameState::IngameUI(IngameUI::ComboOverview),
					GameState::IngameUI(IngameUI::Settings),
				],
				GameState::iterator().take(100).collect::<Vec<_>>()
			);
		}

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
				command: Option<GameStateCommand>,
				ui: HashSet<IngameUI>,
			}

			impl GameStates for &'_ _GameStates {
				fn command(&self) -> Option<GameStateCommand> {
					self.command
				}

				fn ui(&self) -> &'_ HashSet<IngameUI> {
					&self.ui
				}
			}

			let game_states = &_GameStates {
				command: Some(GameStateCommand::Play),
				ui: HashSet::from([IngameUI::Hud, IngameUI::Settings]),
			};

			assert_eq!(
				HashSet::from([
					GameState::Command(GameStateCommand::Play),
					GameState::IngameUI(IngameUI::Hud),
					GameState::IngameUI(IngameUI::Settings)
				]),
				game_states.iter().collect::<HashSet<_>>()
			);
		}
	}

	mod helper_system {
		use super::*;
		use testing::SingleThreadedApp;

		struct _Plugin;

		impl GameStatesRead for _Plugin {
			type TGameStates = _Param<'static>;
		}

		#[derive(Resource)]
		struct _States {
			command: Option<GameStateCommand>,
			ui: HashSet<IngameUI>,
		}

		#[derive(SystemParam)]
		struct _Param<'w> {
			states: Res<'w, _States>,
		}

		impl GameStates for _Param<'_> {
			fn command(&self) -> Option<GameStateCommand> {
				self.states.command
			}

			fn ui(&self) -> &'_ HashSet<IngameUI> {
				&self.states.ui
			}
		}

		#[derive(Resource, Debug, PartialEq, Default)]
		struct SystemRun(bool);

		impl SystemRun {
			fn check(run: ResMut<Self>) {
				let Self(run) = run.into_inner();

				*run = true;
			}
		}

		fn setup<const N: usize>(command: GameStateCommand, ui: [IngameUI; N]) -> App {
			let mut app = App::new().single_threaded(Update);

			app.init_resource::<SystemRun>();
			app.insert_resource(_States {
				command: Some(command),
				ui: HashSet::from(ui),
			});

			app
		}

		#[test]
		fn run_active() {
			let mut app = setup(GameStateCommand::Play, []);
			app.add_systems(
				Update,
				SystemRun::check.run_if(_Plugin::in_game_state([GameStateCommand::Play])),
			);

			app.update();

			assert_eq!(&SystemRun(true), app.world().resource::<SystemRun>());
		}

		#[test]
		fn do_not_run_if_not_active() {
			let mut app = setup(GameStateCommand::Pause, []);
			app.add_systems(
				Update,
				SystemRun::check.run_if(_Plugin::in_game_state([GameStateCommand::Play])),
			);

			app.update();

			assert_eq!(&SystemRun(false), app.world().resource::<SystemRun>());
		}

		#[test]
		fn run_if_ui_active() {
			let mut app = setup(GameStateCommand::Pause, [IngameUI::Hud]);
			app.add_systems(
				Update,
				SystemRun::check.run_if(_Plugin::in_game_state([IngameUI::Hud])),
			);

			app.update();

			assert_eq!(&SystemRun(true), app.world().resource::<SystemRun>());
		}

		#[test]
		fn do_not_run_ingame_ui_not_active() {
			let mut app = setup(GameStateCommand::Pause, []);
			app.add_systems(
				Update,
				SystemRun::check.run_if(_Plugin::in_game_state([IngameUI::Hud])),
			);

			app.update();

			assert_eq!(&SystemRun(false), app.world().resource::<SystemRun>());
		}
	}
}
